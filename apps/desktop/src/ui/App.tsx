import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import {
  BridgeError,
  type CloseRequest,
  DesktopBridge,
  type LayoutResult,
  type SessionCommand,
  type SessionProjection,
  type TaggedLayout,
} from "../bridge";

const bridge = new DesktopBridge();

type DiscardRequest = "new" | "open";

export default function App() {
  const [projection, setProjection] = useState<SessionProjection | null>(bridge.projection);
  const [layout, setLayout] = useState<TaggedLayout | null>(null);
  const [selectedPulse, setSelectedPulse] = useState(0);
  const [register, setRegister] = useState(0);
  const [zoom, setZoom] = useState(1);
  const [metadataDraft, setMetadataDraft] = useState({ title: "", composer: "" });
  const [discardRequest, setDiscardRequest] = useState<DiscardRequest | null>(null);
  const [closeRequest, setCloseRequest] = useState<CloseRequest | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const cellButtons = useRef(new Map<number, HTMLButtonElement>());
  const selectedRef = useRef(0);
  const queuedMetadata = useRef<string | null>(null);
  const layoutRequest = useRef(0);

  useEffect(() => bridge.subscribe((next) => setProjection(next)), []);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    bridge.listenForCloseRequests((request) => {
      if (active) {
        setCloseRequest(request);
        setError(null);
      }
    }).then((stop) => {
      if (active) unlisten = stop;
      else stop();
    }).catch((cause: unknown) => {
      if (active) setError(errorText(cause));
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let active = true;
    bridge.getState().then((next) => {
      if (active) setProjection(next);
    }).catch((cause: unknown) => {
      if (active) setError(errorText(cause));
    });
    return () => { active = false; };
  }, []);

  const document = projection?.document;
  const view = document?.notationViews[0];
  const domain = document?.timeDomains[0];
  const cycleCount = domain?.kind === "cycle" ? domain.cycleCount : 0;
  const pulseCount = domain?.kind === "cycle" ? domain.cycleLength * domain.cycleCount : 0;
  const unsupportedLayout = layout?.layout.kind === "unsupportedView" ? layout.layout : null;
  const canEdit = !!projection?.capabilities.canEdit && !busy && !unsupportedLayout;
  const viewId = view?.id ?? "";
  const sessionId = projection?.sessionId;
  const revision = projection?.revision;

  const setSelection = useCallback((pulse: number, focus = false) => {
    const next = Math.max(0, Math.min(Math.max(pulseCount - 1, 0), pulse));
    selectedRef.current = next;
    setSelectedPulse(next);
    if (focus) requestAnimationFrame(() => cellButtons.current.get(next)?.focus());
  }, [pulseCount]);

  useEffect(() => {
    const title = document?.metadata.title ?? "";
    const composer = document?.metadata.composer ?? "";
    const incoming = JSON.stringify([title, composer]);
    if (queuedMetadata.current && queuedMetadata.current !== incoming) return;
    if (queuedMetadata.current === incoming) queuedMetadata.current = null;
    setMetadataDraft({ title, composer });
  }, [sessionId, document?.metadata.title, document?.metadata.composer]);

  useEffect(() => {
    setSelection(Math.min(selectedRef.current, Math.max(pulseCount - 1, 0)));
  }, [pulseCount, setSelection]);

  useEffect(() => {
    const requestId = ++layoutRequest.current;
    setLayout(null);
    if (!projection?.capabilities.canRender || !viewId) {
      return;
    }
    bridge.getLayout(viewId).then((next) => {
      if (requestId === layoutRequest.current && next) setLayout(next);
    }).catch((cause: unknown) => {
      if (requestId === layoutRequest.current) setError(errorText(cause));
    });
    return () => { layoutRequest.current += 1; };
  }, [sessionId, revision, projection?.capabilities.canRender, viewId]);

  const currentLayout = layout && projection
    && layout.sessionId === projection.sessionId
    && layout.revision === projection.revision
    ? layout
    : null;
  const cycleCells = useMemo(() => currentLayout?.layout.kind === "svg" ? currentLayout.layout.cells : [], [currentLayout]);

  const enqueue = useCallback((command: SessionCommand, affectedPulse = selectedRef.current) => {
    setError(null);
    void bridge.apply(command).catch((cause: unknown) => {
      setSelection(affectedPulse, true);
      setError(errorText(cause));
    });
  }, [setSelection]);

  const commitMetadata = useCallback(async () => {
    const current = bridge.projection;
    if (!current || !current.capabilities.canEdit || unsupportedLayout) return;
    const { title, composer } = metadataDraft;
    const key = JSON.stringify([title, composer]);
    if (queuedMetadata.current === key) return;
    if (
      queuedMetadata.current === null
      && title === current.document.metadata.title
      && composer === current.document.metadata.composer
    ) return;
    queuedMetadata.current = key;
    try {
      await bridge.apply({ kind: "setMetadata", title, composer });
    } finally {
      if (queuedMetadata.current === key) queuedMetadata.current = null;
    }
  }, [metadataDraft, unsupportedLayout]);

  const withBusy = useCallback(async (operation: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await operation();
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      setBusy(false);
    }
  }, []);

  const save = useCallback((saveAs: boolean) => withBusy(async () => {
    await commitMetadata();
    await bridge.flush();
    const result = await bridge.saveDocument(saveAs);
    setNotice(result ? "Composition saved." : "Save cancelled.");
  }), [commitMetadata, withBusy]);

  const requestReplacement = useCallback(async (request: DiscardRequest) => {
    try {
      await commitMetadata();
      await bridge.flush();
      if (bridge.projection?.dirty) {
        setDiscardRequest(request);
      } else {
        await performReplacement(request, false);
      }
    } catch (cause) {
      setError(errorText(cause));
    }
  }, [commitMetadata]);

  const performReplacement = useCallback(async (request: DiscardRequest, discardUnsaved: boolean) => withBusy(async () => {
    const next = request === "new"
      ? await bridge.newDocument(discardUnsaved)
      : await bridge.openDocument(discardUnsaved);
    setDiscardRequest(null);
    setLayout(null);
    setSelection(0);
    setNotice(next ? (request === "new" ? "New composition." : "Composition opened.") : "Open cancelled.");
  }), [setSelection, withBusy]);

  const exportSvg = useCallback(() => withBusy(async () => {
    await bridge.flush();
    const result = await bridge.exportSvg(viewId);
    setNotice(result.status === "cancelled" ? "SVG export cancelled." : "SVG exported.");
  }), [viewId, withBusy]);

  const resolveClose = useCallback((action: "save" | "discard" | "cancel") => withBusy(async () => {
    if (!closeRequest) return;
    if (action === "save") {
      await commitMetadata();
      await bridge.flush();
      const saved = await bridge.saveDocument(false);
      if (!saved) {
        setNotice("Save cancelled. The composition is still open.");
        return;
      }
    }
    const result = await bridge.resolveCloseRequest(closeRequest, action);
    if (action === "cancel") {
      setCloseRequest(null);
      return;
    }
    if (result.status === "closeAuthorized") await bridge.closeCurrentWindow();
  }), [closeRequest, commitMetadata, withBusy]);

  const editCell = useCallback((value: { kind: "degree"; degree: number; register: number } | { kind: "rest" }) => {
    if (!canEdit || !projection) return;
    const pulse = selectedRef.current;
    enqueue({
      kind: "setCell",
      voiceId: projection.document.voices[0].id,
      pulseIndex: pulse,
      value,
    }, pulse);
    if (pulse < pulseCount - 1) setSelection(pulse + 1);
  }, [canEdit, enqueue, projection, pulseCount, setSelection]);

  const clearCell = useCallback(() => {
    if (!canEdit || !projection) return;
    const pulse = selectedRef.current;
    enqueue({
      kind: "clearCell",
      voiceId: projection.document.voices[0].id,
      pulseIndex: pulse,
    }, pulse);
  }, [canEdit, enqueue, projection]);

  const handleCellKey = useCallback((event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight" || event.key === "ArrowUp" || event.key === "ArrowDown") {
      event.preventDefault();
      const delta = event.key === "ArrowLeft" ? -1
        : event.key === "ArrowRight" ? 1
          : event.key === "ArrowUp" ? -16 : 16;
      setSelection(selectedRef.current + delta, true);
      return;
    }
    if (!canEdit || event.altKey || event.ctrlKey || event.metaKey) return;
    if (/^[1-7]$/.test(event.key)) {
      event.preventDefault();
      editCell({ kind: "degree", degree: Number(event.key), register });
    } else if (event.key === ".") {
      event.preventDefault();
      editCell({ kind: "rest" });
    } else if (event.key === "Delete" || event.key === "Backspace") {
      event.preventDefault();
      clearCell();
    }
  }, [canEdit, clearCell, editCell, register, setSelection]);

  useEffect(() => {
    const handleKeys = (event: globalThis.KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        void save(false);
      }
    };
    window.addEventListener("keydown", handleKeys);
    return () => window.removeEventListener("keydown", handleKeys);
  }, [save]);

  const addCycle = () => {
    if (canEdit && domain?.kind === "cycle") enqueue({ kind: "addCycle", domainId: domain.id });
  };

  const layoutResult: LayoutResult | null = currentLayout?.layout ?? null;
  const supportedLayout = layoutResult?.kind === "svg" ? layoutResult : null;
  const safeSvg = supportedLayout && isSafeRustSvg(supportedLayout.svg) ? supportedLayout.svg : null;
  const layoutError = supportedLayout && !safeSvg ? "Rust returned SVG outside the Q1 safe-layout contract." : null;

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <h1>Sandikala</h1>
          <span className="brand-divider" aria-hidden="true" />
          <span className="workspace-label">Composition</span>
        </div>
        <div className="document-status" aria-live="polite">
          {projection ? <><span className={projection.dirty ? "status-dot is-dirty" : "status-dot"} />{projection.dirty ? "Unsaved changes" : "Saved"}</> : "Opening session"}
        </div>
      </header>

      <section className="command-bar" aria-label="Document actions">
        <div className="command-group">
          <button type="button" onClick={() => void requestReplacement("new")} disabled={busy} title="New composition">New</button>
          <button type="button" onClick={() => void requestReplacement("open")} disabled={busy} title="Open a composition">Open</button>
        </div>
        <span className="toolbar-separator" aria-hidden="true" />
        <div className="command-group">
        <button type="button" onClick={() => void save(false)} disabled={busy || !projection?.capabilities.canEdit || !!unsupportedLayout} title="Save composition">Save</button>
          <button type="button" onClick={() => void save(true)} disabled={busy || !projection} title="Save composition as">Save As</button>
          <button type="button" onClick={exportSvg} disabled={busy || !projection?.capabilities.canRender || !!unsupportedLayout} title="Export the current sheet as SVG">Export SVG</button>
        </div>
        <span className="toolbar-separator" aria-hidden="true" />
        <div className="command-group history-actions">
          <button type="button" onClick={() => enqueue({ kind: "undo" })} disabled={!canEdit || !projection?.canUndo} aria-label="Undo" title="Undo">↶</button>
          <button type="button" onClick={() => enqueue({ kind: "redo" })} disabled={!canEdit || !projection?.canRedo} aria-label="Redo" title="Redo">↷</button>
        </div>
        <span className="toolbar-spacer" />
        {projection && <span className="revision-label">Revision {projection.revision}</span>}
      </section>

      {error && <div className="alert-line" role="alert"><span>{error}</span><button type="button" onClick={() => setError(null)} aria-label="Dismiss error">×</button></div>}
      {notice && <div className="notice-line" role="status">{notice}</div>}

      <div className="editor-layout">
        <aside className="properties-panel" aria-label="Composition properties">
          <div className="panel-heading">
            <h2>Composition</h2>
            <span className="part-tag">{document?.parts[0]?.name ?? "Loading"}</span>
          </div>
          <label className="field-label" htmlFor="score-title">Title</label>
          <input
            id="score-title"
            value={metadataDraft.title}
            maxLength={256}
            disabled={!canEdit}
            onChange={(event) => setMetadataDraft((draft) => ({ ...draft, title: event.target.value }))}
            onBlur={() => void commitMetadata().catch((cause: unknown) => setError(errorText(cause)))}
          />
          <label className="field-label" htmlFor="score-composer">Composer</label>
          <input
            id="score-composer"
            value={metadataDraft.composer}
            maxLength={256}
            disabled={!canEdit}
            onChange={(event) => setMetadataDraft((draft) => ({ ...draft, composer: event.target.value }))}
            onBlur={() => void commitMetadata().catch((cause: unknown) => setError(errorText(cause)))}
          />

          <div className="control-section">
            <div className="section-heading">
              <h3>Register</h3>
              <span className="control-value">{register > 0 ? "Upper" : register < 0 ? "Lower" : "Base"}</span>
            </div>
            <div className="segmented-control" role="group" aria-label="Register">
              <button type="button" className={register === -1 ? "is-active" : ""} onClick={() => setRegister(-1)} aria-pressed={register === -1} disabled={busy || !projection?.capabilities.canEdit}>Lower</button>
              <button type="button" className={register === 0 ? "is-active" : ""} onClick={() => setRegister(0)} aria-pressed={register === 0} disabled={busy || !projection?.capabilities.canEdit}>Base</button>
              <button type="button" className={register === 1 ? "is-active" : ""} onClick={() => setRegister(1)} aria-pressed={register === 1} disabled={busy || !projection?.capabilities.canEdit}>Upper</button>
            </div>
          </div>

          <div className="control-section">
            <div className="section-heading"><h3>Enter</h3><span className="control-value">Cell {selectedPulse + 1}</span></div>
            <div className="degree-pad" role="group" aria-label="Enter degree">
              {Array.from({ length: 7 }, (_, index) => index + 1).map((degree) => (
                <button type="button" key={degree} onClick={() => editCell({ kind: "degree", degree, register })} disabled={!canEdit} aria-label={`Enter degree ${degree}`}>{degree}</button>
              ))}
              <button type="button" className="rest-key" onClick={() => editCell({ kind: "rest" })} disabled={!canEdit} aria-label="Enter rest">.</button>
            </div>
            <button type="button" className="clear-cell" onClick={clearCell} disabled={!canEdit}>Clear selected cell</button>
          </div>

          <div className="control-section cycle-section">
            <div className="section-heading"><h3>Cycles</h3><span className="control-value">{cycleCount}</span></div>
            <button type="button" className="add-cycle" onClick={addCycle} disabled={!canEdit || cycleCount >= 64}>Add Cycle <span aria-hidden="true">+</span></button>
          </div>
        </aside>

        <section className="score-workspace" aria-label="Numerical score editor">
          <div className="score-toolbar">
            <div className="score-title">
              <span className="eyebrow">SHEET</span>
              <strong>{document?.metadata.title || "Untitled"}</strong>
              {document?.metadata.composer && <span>{document.metadata.composer}</span>}
            </div>
            <div className="viewport-controls" aria-label="Sheet view controls">
              <button type="button" onClick={() => setZoom((value) => Math.max(0.5, Math.round((value - 0.1) * 10) / 10))} disabled={zoom <= 0.5} aria-label="Zoom out" title="Zoom out">−</button>
              <output aria-label="Zoom level">{Math.round(zoom * 100)}%</output>
              <button type="button" onClick={() => setZoom((value) => Math.min(2.5, Math.round((value + 0.1) * 10) / 10))} disabled={zoom >= 2.5} aria-label="Zoom in" title="Zoom in">+</button>
              <button type="button" onClick={() => setZoom(1)} disabled={zoom === 1} title="Reset zoom">Reset</button>
            </div>
          </div>

          {!projection && <div className="empty-state" role="status">Connecting to the desktop session…</div>}
          {projection && (!projection.capabilities.canRender || unsupportedLayout) && (
            <div className="unsupported-state" role="status">
              <div className="unsupported-mark" aria-hidden="true">i</div>
              <h2>This document is read-only in Q1</h2>
              <p>{unsupportedLayout?.reason ?? projection.capabilities.reason ?? "This notation view is not supported by the current editor."}</p>
              <p>The source document remains unchanged. Use Save As to keep a copy.</p>
            </div>
          )}
          {projection?.capabilities.canRender && !unsupportedLayout && (
            <div className="viewport" aria-label="Scrollable sheet" tabIndex={0}>
              {safeSvg && supportedLayout ? (
                <div
                  className="score-canvas"
                  style={{
                    width: supportedLayout.width * zoom,
                    height: supportedLayout.height * zoom,
                    transform: "none",
                  }}
                >
                  <div
                    className="sheet-svg"
                    style={{ width: supportedLayout.width, height: supportedLayout.height, transform: `scale(${zoom})` }}
                    role="img"
                    aria-label={`Numerical sheet for ${document?.metadata.title || "Untitled"}`}
                    dangerouslySetInnerHTML={{ __html: safeSvg }}
                  />
                  <div className="cell-overlay" style={{ width: supportedLayout.width, height: supportedLayout.height, transform: `scale(${zoom})` }}>
                    {cycleCells.map((cell) => (
                      <button
                        key={cell.pulseIndex}
                        type="button"
                        ref={(node) => {
                          if (node) cellButtons.current.set(cell.pulseIndex, node);
                          else cellButtons.current.delete(cell.pulseIndex);
                        }}
                        className={cell.pulseIndex === selectedPulse ? "score-cell is-selected" : "score-cell"}
                        style={{ left: cell.x, top: cell.y, width: cell.width, height: cell.height }}
                        aria-label={`Pulse ${cell.pulseIndex + 1}, ${cell.eventId ? "occupied" : "blank"}`}
                        aria-pressed={cell.pulseIndex === selectedPulse}
                        tabIndex={cell.pulseIndex === selectedPulse ? 0 : -1}
                        onFocus={() => setSelection(cell.pulseIndex)}
                        onClick={() => setSelection(cell.pulseIndex)}
                        onKeyDown={handleCellKey}
                      />
                    ))}
                  </div>
                </div>
              ) : layoutError ? <div className="empty-state" role="alert">{layoutError}</div>
                : <div className="empty-state" role="status">Rendering sheet…</div>}
            </div>
          )}
        </section>
      </div>

      {discardRequest && (
        <div className="modal-scrim">
          <section className="discard-dialog" role="alertdialog" aria-modal="true" aria-labelledby="discard-title" aria-describedby="discard-copy">
            <div className="dialog-mark" aria-hidden="true">!</div>
            <h2 id="discard-title">Discard unsaved changes?</h2>
            <p id="discard-copy">Your edits to this composition have not been saved.</p>
            <div className="dialog-actions">
              <button type="button" onClick={() => setDiscardRequest(null)} autoFocus>Keep editing</button>
              <button type="button" className="danger-action" onClick={() => void performReplacement(discardRequest, true)}>Discard and {discardRequest === "new" ? "create" : "open"}</button>
            </div>
          </section>
        </div>
      )}

      {closeRequest && (
        <div className="modal-scrim">
          <section className="discard-dialog" role="alertdialog" aria-modal="true" aria-labelledby="close-title" aria-describedby="close-copy">
            <div className="dialog-mark" aria-hidden="true">!</div>
            <h2 id="close-title">Close this composition?</h2>
            <p id="close-copy">There are unsaved changes in this document.</p>
            <div className="dialog-actions">
              <button type="button" onClick={() => void resolveClose("cancel")} disabled={busy} autoFocus>Keep editing</button>
              <button type="button" onClick={() => void resolveClose("discard")} disabled={busy} className="danger-action">Discard and close</button>
              <button type="button" onClick={() => void resolveClose("save")} disabled={busy}>Save and close</button>
            </div>
          </section>
        </div>
      )}

      <footer className="statusbar">
        <span>{projection?.capabilities.canEdit ? "Saron · Numerical demo" : "Read-only document"}</span>
        <span>{projection ? `Revision ${projection.revision}` : ""}</span>
        <span className="queue-status" aria-live="polite">{bridge.pendingActions ? `${bridge.pendingActions} edit${bridge.pendingActions === 1 ? "" : "s"} pending` : ""}</span>
      </footer>
    </main>
  );
}

function isSafeRustSvg(svg: string): boolean {
  return !/<\s*(script|foreignObject)\b|\son[a-z]+\s*=|(?:href|src)\s*=\s*["'](?:https?:|data:|\/\/)|url\(\s*["']?https?:/i.test(svg);
}

function errorText(cause: unknown): string {
  if (cause instanceof BridgeError) return cause.message;
  return cause instanceof Error ? cause.message : String(cause);
}
