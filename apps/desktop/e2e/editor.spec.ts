import { expect, test } from "@playwright/test";

test("Q1 editor inserts, saves, reopens, and exports through the Tauri bridge", async ({ page }) => {
  await page.addInitScript(() => {
    type MockCommand = {
      kind: string;
      voiceId?: string;
      pulseIndex?: number;
      value?: { kind: string; degree?: number; register?: number };
      title?: string;
      composer?: string;
    };

    const documentState = {
      schemaVersion: "0.1.0-q1",
      id: "doc-q1",
      metadata: { title: "Untitled", composer: "" },
      parts: [{ id: "part-1", name: "Saron", voiceIds: ["voice-1"] }],
      voices: [{ id: "voice-1", partId: "part-1", timeDomainId: "domain-1" }],
      timeDomains: [{ kind: "cycle", id: "domain-1", unit: "pulse", cycleLength: 16, cycleCount: 1 }],
      pitchSystems: [{ kind: "degree", id: "context-1" }],
      events: [] as Array<{ id: string; voiceId: string; kind: string; pulseIndex: number; degree?: number; register?: number }>,
      relations: [],
      notationViews: [{ id: "view-1", providerId: "sandikala.numeric-demo.v1", partIds: ["part-1"], settings: {} }],
      extensions: {},
    };

    type MockProjection = {
      sessionId: string;
      document: typeof documentState;
      revision: number;
      dirty: boolean;
      canUndo: boolean;
      canRedo: boolean;
      capabilities: { canEdit: boolean; canRender: boolean; reason: string | null };
    };

    const initialProjection: MockProjection = {
      sessionId: "session-q1",
      document: documentState,
      revision: 0,
      dirty: false,
      canUndo: false,
      canRedo: false,
      capabilities: { canEdit: true, canRender: true, reason: null },
    };
    let state = structuredClone(initialProjection);
    let saved = structuredClone(initialProjection);
    let callbackId = 0;

    const copy = () => structuredClone(state);
    const internals = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
      transformCallback: (_callback: (data: unknown) => void) => ++callbackId,
      unregisterCallback: (_id: number) => undefined,
      invoke: async (command: string, args: Record<string, unknown> = {}): Promise<unknown> => {
        if (command === "plugin:event|listen") return args.handler;
        if (command === "plugin:event|unlisten") return null;
        if (command === "get_state") return copy();
        if (command === "get_layout") {
          const cells = Array.from({ length: 16 }, (_, pulseIndex) => ({
            voiceId: "voice-1",
            pulseIndex,
            eventId: state.document.events.find((event) => event.pulseIndex === pulseIndex)?.id ?? null,
            x: 16 + pulseIndex * 38,
            y: 38,
            width: 34,
            height: 44,
          }));
          return {
            sessionId: state.sessionId,
            revision: state.revision,
            layout: {
              kind: "svg",
              providerId: "sandikala.numeric-demo.v1",
              width: 640,
              height: 120,
              svg: '<svg xmlns="http://www.w3.org/2000/svg" width="640" height="120"><line x1="8" y1="60" x2="632" y2="60" stroke="black" stroke-width="2"/></svg>',
              cells,
            },
          };
        }
        if (command === "apply_command") {
          const edit = args.command as MockCommand;
          state.revision += 1;
          state.dirty = true;
          state.canUndo = true;
          if (edit.kind === "setMetadata") {
            state.document.metadata = { title: edit.title ?? "", composer: edit.composer ?? "" };
          } else if (edit.kind === "setCell" && edit.pulseIndex !== undefined) {
            state.document.events = state.document.events.filter((event) => event.pulseIndex !== edit.pulseIndex);
            state.document.events.push({
              id: `event-${state.revision}`,
              voiceId: edit.voiceId ?? "voice-1",
              kind: edit.value?.kind === "rest" ? "rest" : "sound",
              pulseIndex: edit.pulseIndex,
              degree: edit.value?.degree,
              register: edit.value?.register,
            });
          } else if (edit.kind === "clearCell" && edit.pulseIndex !== undefined) {
            state.document.events = state.document.events.filter((event) => event.pulseIndex !== edit.pulseIndex);
          }
          return copy();
        }
        if (command === "save_document") {
          state.dirty = false;
          saved = copy();
          return copy();
        }
        if (command === "open_document") {
          state = structuredClone(saved);
          state.sessionId = "session-reopened";
          return copy();
        }
        if (command === "export_svg") return { status: "exported", path: "q1.svg" };
        throw new Error(`Unexpected Tauri command: ${command}`);
      },
    };

    (window as unknown as Window & { __TAURI_INTERNALS__: typeof internals }).__TAURI_INTERNALS__ = internals;
    (window as unknown as Window & { __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener: (_event: string, _id: number) => void } })
      .__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => undefined };
  });

  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Sandikala" })).toBeVisible();

  await page.getByRole("textbox", { name: "Title" }).fill("Q1 rehearsal");
  await page.getByRole("textbox", { name: "Title" }).press("Tab");
  await page.getByRole("button", { name: "Pulse 1, blank" }).click();
  await page.getByRole("button", { name: "Enter degree 3" }).click();
  await expect(page.getByRole("button", { name: "Pulse 1, occupied" })).toBeVisible();
  await expect(page.getByText("Unsaved changes")).toBeVisible();

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(page.getByRole("status").getByText("Composition saved.")).toBeVisible();
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.getByRole("status").getByText("Composition opened.")).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Title" })).toHaveValue("Q1 rehearsal");
  await expect(page.getByRole("button", { name: "Pulse 1, occupied" })).toBeVisible();

  await page.getByRole("button", { name: "Export SVG", exact: true }).click();
  await expect(page.getByRole("status").getByText("SVG exported.")).toBeVisible();
});
