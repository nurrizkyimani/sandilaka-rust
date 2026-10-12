import { describe, expect, it } from "vitest";
import { BridgeError, DesktopBridge, type SessionProjection } from "./bridge";

function projection(revision = 0): SessionProjection {
  return {
    sessionId: "session-1",
    revision,
    dirty: revision > 0,
    canUndo: revision > 0,
    canRedo: false,
    capabilities: { canEdit: true, canRender: true, reason: null },
    document: {
      schemaVersion: "0.1.0-q1",
      id: "doc-1",
      metadata: { title: "Untitled", composer: "" },
      parts: [{ id: "part-1", name: "Saron", voiceIds: ["voice-1"] }],
      voices: [{ id: "voice-1", partId: "part-1", timeDomainId: "domain-1" }],
      timeDomains: [{ kind: "cycle", id: "domain-1", unit: "pulse", cycleLength: 16, cycleCount: 1 }],
      pitchSystems: [{ kind: "degree", id: "context-1" }],
      events: [],
      relations: [],
      notationViews: [{ id: "view-1", providerId: "sandikala.numeric-demo.v1", partIds: ["part-1"], settings: {} }],
      extensions: {},
    },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

describe("DesktopBridge", () => {
  it("plays the acknowledged score after queued edits and resolves a saved sound path", async () => {
    const gate = deferred<void>();
    let state = projection();
    let playedRevision: unknown;
    const call = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
      if (command === "get_state") return state as T;
      if (command === "find_saron_soundfont") {
        expect(args?.preferredPath).toBe("/saved/saron.sf2");
        return "/library/saron.sf2" as T;
      }
      if (command === "apply_command") {
        await gate.promise;
        state = projection(1);
        return state as T;
      }
      if (command === "play_saron") {
        playedRevision = args?.expectedRevision;
        expect(args?.soundfontPath).toBe("/library/saron.sf2");
        return undefined as T;
      }
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();
    const path = await bridge.findSaronSoundFont("/saved/saron.sf2");
    const edit = bridge.apply({ kind: "setMetadata", title: "Playback", composer: "" });
    const play = bridge.playSaron(path!, 120);
    expect(playedRevision).toBeUndefined();
    gate.resolve(undefined);
    await Promise.all([edit, play]);
    expect(playedRevision).toBe(1);
  });
  it("serializes edits against each acknowledged revision", async () => {
    const gate = deferred<void>();
    let state = projection();
    const revisions: number[] = [];
    const call = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
      if (command === "get_state") return state as T;
      if (command === "apply_command") {
        revisions.push(args?.expectedRevision as number);
        if (revisions.length === 1) await gate.promise;
        state = projection(state.revision + 1);
        return state as T;
      }
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();

    const first = bridge.apply({ kind: "setMetadata", title: "A", composer: "" });
    const second = bridge.apply({ kind: "setCell", voiceId: "voice-1", pulseIndex: 0, value: { kind: "rest" } });
    await Promise.resolve();
    expect(revisions).toEqual([0]);
    gate.resolve(undefined);
    await Promise.all([first, second]);
    expect(revisions).toEqual([0, 1]);
    expect(bridge.projection?.revision).toBe(2);
    expect(bridge.pendingActions).toBe(0);
  });

  it("reports Busy at 32 pending actions instead of dropping an edit", async () => {
    const gate = deferred<void>();
    let state = projection();
    const call = async <T>(command: string): Promise<T> => {
      if (command === "get_state") return state as T;
      if (command === "apply_command") {
        if (state.revision === 0) await gate.promise;
        state = projection(state.revision + 1);
        return state as T;
      }
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();
    const edits = Array.from({ length: 32 }, (_, pulseIndex) =>
      bridge.apply({ kind: "clearCell", voiceId: "voice-1", pulseIndex }));
    await expect(bridge.apply({ kind: "clearCell", voiceId: "voice-1", pulseIndex: 32 }))
      .rejects.toMatchObject({ code: "Busy" });
    expect(bridge.pendingActions).toBe(32);
    gate.resolve(undefined);
    await Promise.all(edits);
    expect(bridge.pendingActions).toBe(0);
  });

  it("rejects a late layout tagged with an acknowledged older revision", async () => {
    const layoutGate = deferred<unknown>();
    let state = projection();
    const call = async <T>(command: string): Promise<T> => {
      if (command === "get_state") return state as T;
      if (command === "get_layout") return layoutGate.promise as Promise<T>;
      if (command === "apply_command") {
        state = projection(state.revision + 1);
        return state as T;
      }
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();
    const request = bridge.getLayout("view-1");
    const edit = bridge.apply({ kind: "setMetadata", title: "Changed", composer: "" });
    await edit;
    layoutGate.resolve({
      sessionId: "session-1",
      revision: 0,
      layout: { kind: "unsupportedView", viewId: "view-1", reason: "test" },
    });
    expect(await request).toBeNull();
  });

  it("binds discard authorization to the revision shown in the close prompt", async () => {
    const argsSeen: Record<string, unknown>[] = [];
    const call = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
      if (command === "get_state") return projection(1) as T;
      if (command === "resolve_close_request") {
        argsSeen.push(args ?? {});
        return { status: "closeAuthorized" } as T;
      }
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();
    await bridge.resolveCloseRequest({
      requestId: "request-1",
      sessionId: "session-1",
      revision: 0,
    }, "discard");

    expect(argsSeen[0].expectedRevision).toBe(0);
  });

  it("refreshes the session if a discarded open dialog returns cancelled", async () => {
    let state = projection();
    const call = async <T>(command: string): Promise<T> => {
      if (command === "get_state") return state as T;
      if (command === "apply_command") {
        state = projection(state.revision + 1);
        return state as T;
      }
      if (command === "open_document") return null as T;
      throw new Error(`unexpected command ${command}`);
    };
    const bridge = new DesktopBridge(call);
    await bridge.getState();
    const edit = bridge.apply({ kind: "setMetadata", title: "Changed", composer: "" });
    const opening = bridge.openDocument(true);
    await expect(edit).rejects.toBeInstanceOf(BridgeError);
    expect(await opening).toBeNull();
    expect(bridge.projection?.revision).toBe(1);
    expect(bridge.projection?.dirty).toBe(true);
  });
});
