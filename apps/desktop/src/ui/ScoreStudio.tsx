import { useEffect, useRef, useState } from "react";
import type { DesktopBridge, PageLayout, Rational, ScoreSettings, SessionProjection, StudioCell, StudioPitch } from "../bridge";
import "./studio.css";

interface Props {
  bridge:DesktopBridge; projection:SessionProjection; busy:boolean; error:string|null; notice:string|null;
  soundReady:boolean; playing:boolean; playbackPulse:number|null;
  paused:boolean;onPause:()=>void;
  onNew:()=>void;onOpen:()=>void;onSave:(saveAs:boolean)=>Promise<void>;onPlay:()=>void;onStop:()=>void;
  onExportAudio:()=>Promise<unknown>;
}
const catalog=[['saron','Saron'],['gender','Gendèr'],['gambang','Gambang'],['siter','Siter'],['rebab','Rebab'],['suling','Suling'],['bonang','Bonang Barung'],['piano','Piano']];
const val=(r:Rational)=>r.numerator/r.denominator;
const rational=(value:number):Rational=>{let numerator=Math.round(value*4);let denominator=4;while(numerator%2===0&&denominator>1){numerator/=2;denominator/=2;}return{numerator,denominator};};

export default function ScoreStudio(props:Props){
  const {bridge,projection}=props;const doc=projection.document;
  const cfg=doc.extensions['sandikala.studio.v1'] as ScoreSettings;
  const [draft,setDraft]=useState(cfg);
  const [partId,setPartId]=useState<string|null>(null);
  const [layout,setLayout]=useState<PageLayout|null>(null);
  const [selected,setSelected]=useState<{voiceId:string;position:Rational}>({voiceId:doc.voices[0].id,position:rational(0)});
  const [register,setRegister]=useState(0);const [duration,setDuration]=useState(1);
  const [octave,setOctave]=useState(4);const [alteration,setAlteration]=useState(0);
  const [zoom,setZoom]=useState(0.85);const [pending,setPending]=useState(false);
  const [localError,setLocalError]=useState<string|null>(null);
  const [instrument,setInstrument]=useState('gender');
  const [title,setTitle]=useState(doc.metadata.title);const [composer,setComposer]=useState(doc.metadata.composer);
  const [clipboard,setClipboard]=useState<{pitch:StudioPitch|null;extent:Rational}|null>(null);
  const canvas=useRef<HTMLDivElement>(null);
  const selectionRef=useRef(selected);
  const select=(value:typeof selected)=>{selectionRef.current=value;setSelected(value);};
  const disabled=pending||props.busy||!projection.capabilities.canEdit;
  const selectedPart=doc.parts.find(p=>p.voiceIds.includes(selected.voiceId));
  const piano=selectedPart?.instrumentRef?.id==='piano';
  useEffect(()=>{if(piano)setOctave(selectedPart?.voiceIds.indexOf(selected.voiceId)===1?3:4);},[selected.voiceId]);
  const selectedEvent=doc.events.find(e=>e.voiceId===selected.voiceId && val((e.anchor as {position:Rational}).position)===val(selected.position));
  const [mixDraft,setMixDraft]=useState({gain:100,pan:0,mute:false,solo:false});
  const [marksDraft,setMarksDraft]=useState({text:'',velocity:100});
  useEffect(()=>{
    const mixes=doc.extensions['sandikala.mixer.v1'] as Record<string,typeof mixDraft>|undefined;
    setMixDraft(mixes?.[selectedPart?.id??'']??{gain:100,pan:0,mute:false,solo:false});
    const extensions=selectedEvent?.extensions as Record<string,typeof marksDraft>|undefined;
    setMarksDraft(extensions?.['sandikala.event-marks.v1']??{text:'',velocity:100});
  },[selectedPart?.id,selectedEvent?.id,projection.revision]);
  useEffect(()=>{setDraft(cfg);},[projection.sessionId,projection.revision]);
  useEffect(()=>{setTitle(doc.metadata.title);setComposer(doc.metadata.composer);},[doc.metadata.title,doc.metadata.composer]);
  useEffect(()=>{
    if(!doc.voices.some(v=>v.id===selected.voiceId))select({voiceId:doc.voices[0].id,position:rational(0)});
    if(partId&&!doc.parts.some(p=>p.id===partId))setPartId(null);
  },[projection.revision,selected.voiceId,partId]);
  useEffect(()=>{
    let active=true;setLayout(null);
    void bridge.getStudioLayout(partId).then(result=>{if(active&&result?.layout.kind==='pages')setLayout(result.layout.layout);})
      .catch(error=>{if(active)setLocalError(String(error));});
    return()=>{active=false;};
  },[bridge,projection.sessionId,projection.revision,partId]);
  const run=async(action:()=>Promise<unknown>)=>{
    setPending(true);setLocalError(null);
    try{await action();}catch(error){setLocalError(error instanceof Error?error.message:String(error));}finally{setPending(false);}
  };
  const enter=async(pitch:StudioPitch|null)=>{
    const target=selectionRef.current;setLocalError(null);
    const next=val(target.position)+duration;
    if(next<cfg.measures*cfg.numerator*4/cfg.denominator)select({...target,position:rational(next)});
    requestAnimationFrame(()=>canvas.current?.focus());
    try{await bridge.apply({kind:'studioEvent',voiceId:target.voiceId,position:target.position,extent:rational(duration),pitch});}
    catch(error){select(target);setLocalError(error instanceof Error?error.message:String(error));}
  };
  const degree=(number:number)=>{
    const context=doc.pitchSystems.find(s=>s.kind==='degree');if(!context)return;
    void enter({kind:'degree',contextId:context.id,degree:String(number),register});
  };
  const note=(letter:string)=>{
    const context=doc.pitchSystems.find(s=>s.kind==='equalTemperament');if(!context)return;
    void enter({kind:'spelled',contextId:context.id,letter,octave,alteration});
  };
  const clear=()=>run(()=>bridge.apply({kind:'studioClear',voiceId:selected.voiceId,position:selected.position}));
  const copy=()=>{
    if(!selectedEvent)return;
    setClipboard({pitch:selectedEvent.kind==='sound'?(selectedEvent.payload as {pitch:StudioPitch}).pitch:null,extent:selectedEvent.extent as Rational});
  };
  const paste=()=>{
    if(!clipboard)return;
    const isSpelled=clipboard.pitch?.kind==='spelled';
    if(clipboard.pitch&&piano!==isSpelled){setLocalError('Paste into a staff with the same notation and pitch context.');return;}
    void run(()=>bridge.apply({kind:'studioEvent',voiceId:selected.voiceId,position:selected.position,extent:clipboard.extent,pitch:clipboard.pitch}));
  };
  const key=(event:React.KeyboardEvent)=>{
    if(disabled)return;
    if(event.key==='ArrowLeft'||event.key==='ArrowRight'){
      event.preventDefault();const next=val(selected.position)+(event.key==='ArrowLeft'?-1:1)/cfg.subdivision;
      if(next>=0&&next<cfg.measures*cfg.numerator*4/cfg.denominator)select({...selected,position:rational(next)});return;
    }
    if(event.metaKey||event.ctrlKey){if(event.key.toLowerCase()==='c'){event.preventDefault();copy();}if(event.key.toLowerCase()==='v'){event.preventDefault();paste();}return;}
    if(event.key==='Backspace'||event.key==='Delete'){event.preventDefault();void clear();}
    else if(event.key==='.'||event.key==='0'){event.preventDefault();void enter(null);}
    else if(!piano&&/^[1-7]$/.test(event.key)){event.preventDefault();degree(Number(event.key));}
    else if(piano&&/^[a-gA-G]$/.test(event.key)){event.preventDefault();note(event.key.toUpperCase());}
  };
  const saveMetadata=()=>{if(title!==doc.metadata.title||composer!==doc.metadata.composer)void run(()=>bridge.apply({kind:'setMetadata',title,composer}));};
  const saveAll=(saveAs:boolean)=>void run(async()=>{if(title!==doc.metadata.title||composer!==doc.metadata.composer)await bridge.apply({kind:'setMetadata',title,composer});await props.onSave(saveAs);});
  const selectCell=(cell:StudioCell)=>select({voiceId:cell.voiceId,position:cell.position});
  const breakBefore=()=>{const bar=Math.floor(val(selected.position)/(cfg.numerator*4/cfg.denominator));
    if(bar===0){setLocalError('Select a bar after bar 1 to place a system break.');return;}
    void run(()=>bridge.apply({kind:'configureScore',settings:{...cfg,systemBreaks:cfg.systemBreaks.includes(bar)?cfg.systemBreaks.filter(v=>v!==bar):[...cfg.systemBreaks,bar].sort((a,b)=>a-b)}}));
  };
  return <div className="score-studio">
    {layout&&<style>{layout.fontCss}</style>}
    <header className="studio-header"><strong>Sandikala</strong><span className="studio-main-tab">Score</span><button onClick={()=>document.getElementById('studio-publish')?.scrollIntoView({block:'start'})}>Publish</button><span className="studio-spacer"/><span>{doc.metadata.title}</span><span className="studio-spacer"/>
      <button aria-label="Play" title="Play (Space)" onClick={props.playing&&props.paused?props.onPause:props.onPlay} disabled={disabled||(props.playing&&!props.paused)||!props.soundReady}>▶</button>
      <button aria-label={props.paused?'Resume':'Pause'} title="Pause / Resume (Space)" onClick={props.onPause} disabled={disabled||!props.playing}>Ⅱ</button>
      <button aria-label="Stop playback" title="Stop (Space)" onClick={props.onStop} disabled={!props.playing||disabled}>■</button>
      <span>{props.playing?(props.paused?'Paused':'Playing'):props.soundReady?'Sounds ready':'Locate sound library'}</span>
      {props.playing&&<input aria-label="Playback position" type="range" min={0} max={Math.max(1,...doc.events.map(e=>val((e.anchor as {position:Rational}).position)+val(e.extent as Rational)))} step={0.25} value={props.playbackPulse??0} onChange={e=>void run(()=>bridge.seekPlayback(Number(e.target.value)))} disabled={disabled}/>}
    </header>
    <nav className="studio-toolbar" aria-label="Score tools">
      <button onClick={props.onNew} disabled={disabled}>New</button><button onClick={props.onOpen} disabled={disabled}>Open</button>
      <button onClick={()=>saveAll(false)} disabled={disabled}>Save</button>
      <button onClick={()=>saveAll(true)} disabled={disabled}>Save as</button>
      <span className="studio-rule"/>
      <button onClick={()=>void run(()=>bridge.apply({kind:'undo'}))} disabled={disabled||!projection.canUndo}>↶</button>
      <button onClick={()=>void run(()=>bridge.apply({kind:'redo'}))} disabled={disabled||!projection.canRedo}>↷</button>
      <span className="studio-rule"/><span>Duration</span>
      {[4,2,1,0.5,0.25].map(d=><button key={d} aria-pressed={duration===d} className={duration===d?'active':''} onClick={()=>setDuration(d)}>{d===0.5?'½':d===0.25?'¼':d}</button>)}
      <button onClick={()=>void enter(null)} disabled={disabled}>Rest ·</button>
      <button onClick={copy} disabled={!selectedEvent}>Copy</button><button onClick={paste} disabled={disabled||!clipboard}>Paste</button>
      <span className="studio-spacer"/><button onClick={()=>setZoom(Math.max(0.4,zoom-0.1))}>−</button><span>{Math.round(zoom*100)}%</span><button onClick={()=>setZoom(Math.min(1.8,zoom+0.1))}>+</button>
    </nav>
    <div className="studio-workspace">
      <aside className="studio-panel" aria-label="Score properties">
        <h3>Instruments</h3>
        {doc.parts.map(part=><div className="studio-part" key={part.id}><button className={part.id===selectedPart?.id?'active':''} onClick={()=>select({voiceId:part.voiceIds[0],position:selected.position})}>{part.name}</button>
          {part.voiceIds.map((voice,i)=><button key={voice} title={`Select ${part.name} staff ${i+1}`} onClick={()=>select({voiceId:voice,position:selected.position})} className={selected.voiceId===voice?'active':''}>{i+1}</button>)}
          <button title={`Remove ${part.name} and its notes`} onClick={()=>{if(window.confirm(`Remove ${part.name} and its notes? Undo can restore it.`))void run(()=>bridge.apply({kind:'removePart',partId:part.id}));}} disabled={disabled||doc.parts.length===1}>×</button>
        </div>)}
        <select aria-label="Add instrument" value={instrument} onChange={e=>setInstrument(e.target.value)}>{catalog.map(([id,name])=><option key={id} value={id}>{name}</option>)}</select>
        <button onClick={()=>void run(()=>bridge.apply({kind:'addInstrument',instrumentId:instrument}))} disabled={disabled}>+ Add instrument</button>
        <h3>{selectedPart?.name} mixer</h3>
        <label>Volume %<input type="number" min={0} max={100} value={mixDraft.gain} onChange={e=>setMixDraft({...mixDraft,gain:Number(e.target.value)})}/></label>
        <label>Pan (-100 left, +100 right)<input type="number" min={-100} max={100} value={mixDraft.pan} onChange={e=>setMixDraft({...mixDraft,pan:Number(e.target.value)})}/></label>
        <div className="studio-pad"><button aria-pressed={mixDraft.mute} onClick={()=>setMixDraft({...mixDraft,mute:!mixDraft.mute})}>Mute</button><button aria-pressed={mixDraft.solo} onClick={()=>setMixDraft({...mixDraft,solo:!mixDraft.solo})}>Solo</button></div>
        <button onClick={()=>void run(()=>bridge.apply({kind:'setPartMix',partId:selectedPart!.id,...mixDraft}))} disabled={disabled}>Apply mixer</button>
        <h3>{selectedPart?.name} input</h3>
        {piano?<><div className="studio-pad">{'CDEFGAB'.split('').map(letter=><button key={letter} onClick={()=>note(letter)} disabled={disabled}>{letter}</button>)}</div><label>Octave<input type="number" min={1} max={7} value={octave} onChange={e=>setOctave(Number(e.target.value))}/></label>
          <label>Accidental<select value={alteration} onChange={e=>setAlteration(Number(e.target.value))}><option value={-1}>♭ Flat</option><option value={0}>♮ Natural</option><option value={1}>♯ Sharp</option></select></label></>
          :<><div className="studio-pad">{[1,2,3,4,5,6,7].map(n=><button key={n} onClick={()=>degree(n)} disabled={disabled}>{n}</button>)}</div><div className="studio-pad">{[-1,0,1].map(r=><button key={r} onClick={()=>setRegister(r)} className={register===r?'active':''}>{r===-1?'Lower':r===0?'Normal':'Upper'}</button>)}</div></>}
        <button onClick={()=>void clear()} disabled={disabled}>Delete selected event</button>
        <h3>Selected event marks</h3><label>Annotation<input maxLength={24} value={marksDraft.text} onChange={e=>setMarksDraft({...marksDraft,text:e.target.value})}/></label>
        <label>Velocity<input type="number" min={1} max={127} value={marksDraft.velocity} onChange={e=>setMarksDraft({...marksDraft,velocity:Number(e.target.value)})}/></label>
        <button onClick={()=>void run(()=>bridge.apply({kind:'annotateEvent',eventId:selectedEvent!.id,...marksDraft}))} disabled={disabled||!selectedEvent}>Apply event marks</button>
        <h3>Score</h3><label>Title<input value={title} onChange={e=>setTitle(e.target.value)} onBlur={saveMetadata}/></label><label>Composer<input value={composer} onChange={e=>setComposer(e.target.value)} onBlur={saveMetadata}/></label>
        <h3>Meter & layout</h3><label>Beats<input aria-label="Meter numerator" type="number" min={1} max={12} value={draft.numerator} onChange={e=>setDraft({...draft,numerator:Number(e.target.value)})}/></label>
        <label>Beat unit<select value={draft.denominator} onChange={e=>setDraft({...draft,denominator:Number(e.target.value),subdivision:Number(e.target.value)/4})}>{[4,8,16].map(n=><option key={n} value={n}>1/{n}</option>)}</select></label>
        <label>Bars<input type="number" min={1} max={128} value={draft.measures} onChange={e=>setDraft({...draft,measures:Number(e.target.value)})}/></label>
        <label>Empty pulse dots<select value={draft.subdivision} onChange={e=>setDraft({...draft,subdivision:Number(e.target.value)})}><option value={1}>Quarter</option><option value={2}>Eighth</option><option value={4}>Sixteenth</option></select></label>
        <label>Tempo ♩<input type="number" min={30} max={300} value={draft.tempo} onChange={e=>setDraft({...draft,tempo:Number(e.target.value)})}/></label>
        <label>Paper<select value={draft.letterPaper?'letter':'a4'} onChange={e=>setDraft({...draft,letterPaper:e.target.value==='letter'})}><option value="a4">A4</option><option value="letter">Letter</option></select></label>
        <label>Orientation<select value={draft.landscape?'landscape':'portrait'} onChange={e=>setDraft({...draft,landscape:e.target.value==='landscape'})}><option value="portrait">Portrait</option><option value="landscape">Landscape</option></select></label>
        <button onClick={()=>void run(()=>bridge.apply({kind:'configureScore',settings:draft}))} disabled={disabled}>Apply score settings</button>
        <button onClick={breakBefore} disabled={disabled}>Break system before selected bar</button>
        <h3 id="studio-publish">Publish selected view</h3><button onClick={()=>void run(()=>bridge.exportScore(partId,'svg'))} disabled={!layout||disabled}>Export SVG</button>
        <button onClick={()=>void run(props.onExportAudio)} disabled={!props.soundReady||disabled}>Export audio WAV</button>
        <button onClick={()=>void run(()=>bridge.exportScore(partId,'pdf'))} disabled={!layout||disabled}>Export PDF</button>
        <button onClick={()=>void run(()=>bridge.exportScore(partId,'html'))} disabled={!layout||disabled}>Export printable pages</button>
        <button onClick={()=>void run(()=>bridge.printScore())} disabled={!layout||disabled}>Print / Save as PDF</button>
        <p className="studio-help">Click a pulse, then type 1–7 for gamelan or A–G for piano. . enters a rest. Arrow keys select the next pulse. Space toggles playback. Piano uses a synthesized preview; gamelan uses the installed recorded SoundFonts.</p>
      </aside>
      <section className="studio-canvas" aria-label="Score canvas">
        <nav className="studio-tabs"><button className={partId===null?'active':''} onClick={()=>setPartId(null)}>Full score</button>{doc.parts.map(p=><button key={p.id} className={partId===p.id?'active':''} onClick={()=>setPartId(p.id)}>{p.name}</button>)}</nav>
        {(localError||props.error)&&<div role="alert" className="studio-error">{localError||props.error}</div>}
        {props.notice&&<div className="studio-notice">{props.notice}</div>}
        <div className="studio-pages" ref={canvas} tabIndex={0} onKeyDown={key} aria-label="Page view">
          {layout?layout.pages.map(page=><div className="studio-page-slot" key={page.number} style={{width:layout.width*zoom,height:layout.height*zoom}}><div className="studio-page" style={{width:layout.width,height:layout.height,transform:`scale(${zoom})`}}>
            <div className="studio-svg" dangerouslySetInnerHTML={{__html:page.svg}}/>
            <div className="studio-hit-layer">{page.cells.map(cell=><button key={`${cell.voiceId}:${val(cell.position)}`} className={`studio-hit score-cell${selected.voiceId===cell.voiceId&&val(selected.position)===val(cell.position)?' chosen':''}${props.playing&&props.playbackPulse!==null&&val(cell.position)<=props.playbackPulse&&val(cell.endPosition)>props.playbackPulse?' sounding':''}`} style={{left:cell.x,top:cell.y,width:cell.width,height:cell.height}}
              aria-label={`${doc.parts.find(p=>p.voiceIds.includes(cell.voiceId))?.name}, pulse ${val(cell.position)+1}${cell.eventId?', note':''}`} tabIndex={selected.voiceId===cell.voiceId&&val(selected.position)===val(cell.position)?0:-1} onClick={()=>selectCell(cell)} onFocus={()=>selectCell(cell)} disabled={disabled}/>)}</div>
          </div></div>):<p className="studio-rendering">Laying out score pages…</p>}
        </div>
      </section>
    </div>
    <footer className="studio-footer"><span>{selectedPart?.name} · {piano?'Western staff':'Numeric Pelog'} · pulse {val(selected.position)+1}</span><span>{layout?.pages.length??0} pages · Leland / Edwin · revision {projection.revision}</span></footer>
  </div>;
}
