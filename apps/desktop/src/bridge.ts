import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface ScoreDocument {
  schemaVersion: string;
  id: string;
  metadata: { title: string; composer: string };
  parts: Array<{ id: string; name: string; voiceIds: string[] }>;
  voices: Array<{ id: string; partId: string; timeDomainId: string }>;
  timeDomains: Array<
    | { kind: "cycle"; id: string; unit: string; cycleLength: number; cycleCount: number }
    | { kind: "metric"; id: string; unit: string; length: { numerator: number; denominator: number } }
  >;
  pitchSystems: Array<Record<string, unknown> & { kind: string; id: string }>;
  events: Array<Record<string, unknown> & { id: string; voiceId: string; kind: "sound" | "rest" }>;
  relations: unknown[];
  notationViews: Array<{ id: string; providerId: string; partIds: string[]; settings: Record<string, unknown> }>;
  extensions: Record<string, unknown>;
}

export interface SessionProjection {
  sessionId: string;
  document: ScoreDocument;
  revision: number;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
  capabilities: { canEdit: boolean; canRender: boolean; reason: string | null };
}

export type CellValue =
  | { kind: "degree"; degree: number; register: number }
  | { kind: "rest" };

export type SessionCommand =
  | { kind: "setCell"; voiceId: string; pulseIndex: number; value: CellValue }
  | { kind: "clearCell"; voiceId: string; pulseIndex: number }
  | { kind: "addCycle"; domainId: string }
  | { kind: "setMetadata"; title: string; composer: string }
  | { kind: "undo" }
  | { kind: "redo" };

export interface CellGeometry {
  voiceId: string;
  pulseIndex: number;
  eventId: string | null;
  x: number;
  y: number;
  width: number;
  height: number;
}

export type LayoutResult =
  | { kind: "svg"; providerId: string; width: number; height: number; svg: string; cells: CellGeometry[] }
  | { kind: "unsupportedView"; viewId: string; reason: string };

export interface TaggedLayout {
  sessionId: string;
  revision: number;
  layout: LayoutResult;
}

export interface ExportResult {
  status: "exported" | "cancelled";
  path?: string;
}

export interface CloseRequest {
  requestId: string;
  sessionId: string;
  revision: number;
}

export interface CloseResolution {
  status: "closeAuthorized" | "cancelled";
}

type InvokeFunction = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export class BridgeError extends Error {
  constructor(message: string, readonly code = "DesktopError") {
    super(message);
    this.name = "BridgeError";
  }
}

interface PendingAction {
  generation: number;
  command: SessionCommand;
  resolve: (projection: SessionProjection) => void;
  reject: (error: unknown) => void;
}

const ACTION_LIMIT = 32;
const cancelled = () => new BridgeError("This request was replaced by a newer document session.", "StaleSession");

export class DesktopBridge {
  private projectionValue: SessionProjection | null = null;
  private generation = 0;
  private pendingCount = 0;
  private running = false;
  private replacing = false;
  private queue: PendingAction[] = [];
  private idleWaiters: Array<() => void> = [];
  private listeners = new Set<(projection: SessionProjection) => void>();

  constructor(private readonly call: InvokeFunction = invoke) {}

  get projection(): SessionProjection | null {
    return this.projectionValue;
  }

  get pendingActions(): number {
    return this.pendingCount;
  }

  subscribe(listener: (projection: SessionProjection) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  async getState(): Promise<SessionProjection> {
    const generation = this.generation;
    const projection = await this.call<SessionProjection>("get_state");
    if (generation === this.generation) this.publish(projection);
    return projection;
  }

  apply(command: SessionCommand): Promise<SessionProjection> {
    if (this.replacing) {
      return Promise.reject(new BridgeError("A document operation is in progress.", "Busy"));
    }
    if (this.pendingCount >= ACTION_LIMIT) {
      return Promise.reject(new BridgeError("The edit queue is full. Wait for pending edits to finish.", "Busy"));
    }
    return new Promise((resolve, reject) => {
      this.pendingCount += 1;
      this.queue.push({ generation: this.generation, command, resolve, reject });
      void this.drain();
    });
  }

  async flush(): Promise<void> {
    if (this.running || this.queue.length > 0) {
      await new Promise<void>((resolve) => this.idleWaiters.push(resolve));
    }
  }

  async newDocument(discardUnsaved: boolean): Promise<SessionProjection | null> {
    return this.replaceDocument("new_document", { discardUnsaved });
  }

  async openDocument(discardUnsaved: boolean): Promise<SessionProjection | null> {
    return this.replaceDocument("open_document", { discardUnsaved });
  }

  async saveDocument(saveAs: boolean): Promise<SessionProjection | null> {
    if (this.replacing) throw new BridgeError("A document operation is in progress.", "Busy");
    await this.flush();
    const current = this.requireProjection();
    const result = await this.call<SessionProjection | null>("save_document", {
      sessionId: current.sessionId,
      expectedRevision: current.revision,
      saveAs,
    });
    if (result && this.isCurrent(current)) this.publish(result);
    return result;
  }

  async exportSvg(viewId: string): Promise<ExportResult> {
    if (this.replacing) throw new BridgeError("A document operation is in progress.", "Busy");
    await this.flush();
    const current = this.requireProjection();
    return this.call<ExportResult>("export_svg", {
      sessionId: current.sessionId,
      revision: current.revision,
      viewId,
    });
  }

  resolveCloseRequest(request: CloseRequest, action: "save" | "discard" | "cancel"): Promise<CloseResolution> {
    const current = this.requireProjection();
    return this.call<CloseResolution>("resolve_close_request", {
      requestId: request.requestId,
      action,
      sessionId: current.sessionId,
      expectedRevision: action === "discard" ? request.revision : current.revision,
    });
  }

  listenForCloseRequests(handler: (request: CloseRequest) => void): Promise<UnlistenFn> {
    return listen<CloseRequest>("document-close-requested", (event) => handler(event.payload));
  }

  async closeCurrentWindow(): Promise<void> {
    await getCurrentWindow().close();
  }

  async getLayout(viewId: string): Promise<TaggedLayout | null> {
    if (this.replacing) return null;
    await this.flush();
    const current = this.requireProjection();
    const result = await this.call<TaggedLayout>("get_layout", {
      sessionId: current.sessionId,
      revision: current.revision,
      viewId,
    });
    if (!this.isCurrent(current) || result.sessionId !== current.sessionId || result.revision !== current.revision) {
      return null;
    }
    return result;
  }

  private async replaceDocument(
    command: "new_document" | "open_document",
    args: { discardUnsaved: boolean },
  ): Promise<SessionProjection | null> {
    if (this.replacing) throw new BridgeError("A document operation is already in progress.", "Busy");
    this.replacing = true;
    const generation = this.invalidatePending();
    try {
      await this.waitUntilIdle();
      if (generation !== this.generation) return null;
      const result = await this.call<SessionProjection | null>(command, args);
      if (generation !== this.generation) return null;
      if (result) this.publish(result);
      else await this.refreshState(generation);
      return result;
    } catch (error) {
      if (generation === this.generation) await this.refreshState(generation);
      throw toBridgeError(error);
    } finally {
      if (generation === this.generation) this.replacing = false;
    }
  }

  private async refreshState(generation: number): Promise<void> {
    const current = await this.call<SessionProjection>("get_state");
    if (generation === this.generation) this.publish(current);
  }

  private invalidatePending(): number {
    this.generation += 1;
    for (const action of this.queue) action.reject(cancelled());
    this.queue = [];
    this.pendingCount = 0;
    return this.generation;
  }

  private async waitUntilIdle(): Promise<void> {
    if (this.running) await new Promise<void>((resolve) => this.idleWaiters.push(resolve));
  }

  private async drain(): Promise<void> {
    if (this.running) return;
    this.running = true;
    try {
      while (this.queue.length > 0) {
        const action = this.queue.shift()!;
        if (action.generation !== this.generation) {
          action.reject(cancelled());
          continue;
        }
        try {
          const current = this.requireProjection();
          const result = await this.call<SessionProjection>("apply_command", {
            sessionId: current.sessionId,
            expectedRevision: current.revision,
            command: action.command,
          });
          if (action.generation !== this.generation || !this.isCurrent(current)) {
            action.reject(cancelled());
          } else {
            this.publish(result);
            action.resolve(result);
          }
        } catch (error) {
          action.reject(toBridgeError(error));
        } finally {
          if (action.generation === this.generation) this.pendingCount = Math.max(0, this.pendingCount - 1);
        }
      }
    } finally {
      this.running = false;
      const waiters = this.idleWaiters.splice(0);
      waiters.forEach((resolve) => resolve());
    }
  }

  private requireProjection(): SessionProjection {
    if (!this.projectionValue) throw new BridgeError("The document session has not loaded yet.", "NoSession");
    return this.projectionValue;
  }

  private isCurrent(snapshot: SessionProjection): boolean {
    return this.projectionValue?.sessionId === snapshot.sessionId
      && this.projectionValue.revision === snapshot.revision;
  }

  private publish(projection: SessionProjection): void {
    this.projectionValue = projection;
    this.listeners.forEach((listener) => listener(projection));
  }
}

function toBridgeError(error: unknown): BridgeError {
  if (error instanceof BridgeError) return error;
  if (typeof error === "object" && error !== null) {
    const detail = error as { code?: unknown; message?: unknown };
    if (typeof detail.message === "string") {
      return new BridgeError(detail.message, typeof detail.code === "string" ? detail.code : "DesktopError");
    }
  }
  return new BridgeError(typeof error === "string" ? error : "The desktop command failed.");
}
