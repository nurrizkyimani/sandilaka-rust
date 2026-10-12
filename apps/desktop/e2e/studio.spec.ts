import {expect,test} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';

test.beforeAll(()=>{
  execFileSync('rtk',['proxy','cargo','run','-p','sandikala-engine','--example','paged_demo','--','output/pdf'],{cwd:resolve('../..'),timeout:120000});
});
test('paged studio uses real Rust layout, keyboard input, part views and transport',async({page})=>{
  const doc=JSON.parse(readFileSync(resolve('../../output/pdf/mixed-ensemble.sdlk'),'utf8'));
  const full=JSON.parse(readFileSync(resolve('../../output/pdf/layout.json'),'utf8'));
  const piano=JSON.parse(readFileSync(resolve('../../output/pdf/piano-layout.json'),'utf8'));
  await page.addInitScript(({doc,full,piano})=>{
    const state={sessionId:'studio-test',revision:0,dirty:false,canUndo:false,canRedo:false,document:doc,capabilities:{canEdit:true,canRender:true,reason:null}};
    let playing=false;let paused=false;let pulse=0;let callback=0;
    (window as unknown as {studioCalls:unknown[]}).studioCalls=[];
    const internals={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main',windowLabel:'main'}},transformCallback:()=>++callback,unregisterCallback:()=>undefined,
      invoke:async(command:string,args:Record<string,unknown>={})=>{
        (window as unknown as {studioCalls:unknown[]}).studioCalls.push({command,args});
        if(command==='plugin:event|listen')return args.handler;if(command==='plugin:event|unlisten')return null;
        if(command==='get_state')return structuredClone(state);
        if(command==='find_saron_soundfont')return '/library/Gamelan_Saron_Pelog_F3.sf2';
        if(command==='get_layout'||command==='get_studio_layout')return{sessionId:state.sessionId,revision:state.revision,layout:{kind:'pages',layout:args.partId?piano:full}};
        if(command==='apply_command'){
          await new Promise(resolve=>setTimeout(resolve,120));
          const action=args.command as {kind:string;voiceId:string;position:{numerator:number;denominator:number};extent:unknown;pitch:unknown};
          state.revision++;state.dirty=true;state.canUndo=true;
          if(action.kind==='studioEvent'){
            state.document.events=state.document.events.filter((e:{voiceId:string;anchor:{position:{numerator:number;denominator:number}}})=>!(e.voiceId===action.voiceId&&e.anchor.position.numerator/e.anchor.position.denominator===action.position.numerator/action.position.denominator));
            state.document.events.push({id:`edited-${state.revision}`,voiceId:action.voiceId,anchor:{domainId:'main-time',position:action.position},extent:action.extent,kind:action.pitch?'sound':'rest',payload:action.pitch?{pitch:action.pitch}:{},extensions:{}});
          }return structuredClone(state);
        }
        if(command==='play_saron'){playing=true;paused=false;return null;}
        if(command==='stop_saron'){playing=false;paused=false;return null;}
        if(command==='pause_playback'){paused=Boolean(args.paused);return null;}
        if(command==='seek_playback'){pulse=Number(args.pulse);return null;}
        if(command==='saron_playback_status')return{playing,paused,pulsePosition:playing?pulse:null};
        if(command==='save_document'){state.dirty=false;return structuredClone(state);}
        if(command==='export_score'||command==='export_audio')return{status:'exported',path:'preview.pdf'};
        if(command==='print_score')return null;
        throw new Error(`Unexpected command ${command}`);
      }};
    (window as unknown as {__TAURI_INTERNALS__:unknown}).__TAURI_INTERNALS__=internals;
    (window as unknown as {__TAURI_EVENT_PLUGIN_INTERNALS__:unknown}).__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>undefined};
  },{doc,full,piano});
  await page.setViewportSize({width:1800,height:1000});await page.goto('/');
  await expect(page.getByRole('region',{name:'Score canvas'})).toBeVisible();
  await expect(page.locator('.studio-page')).toHaveCount(full.pages.length);
  await page.getByRole('button',{name:'Saron, pulse 1, note',exact:true}).click();
  await page.keyboard.press('2');await page.keyboard.press('3');
  await expect.poll(()=>page.evaluate(()=>(window as unknown as {studioCalls:Array<{command:string;args:{command?:{kind:string;pitch:{degree:string}}}}>}).studioCalls.filter(c=>c.args.command?.kind==='studioEvent').map(c=>c.args.command?.pitch.degree))).toEqual(['2','3']);
  await page.keyboard.press('Space');await expect(page.getByRole('button',{name:'Pause',exact:true})).toBeEnabled();
  await page.keyboard.press('Space');await expect(page.getByRole('button',{name:'Resume',exact:true})).toBeEnabled();
  await page.getByRole('button',{name:'Stop playback'}).click();
  await page.locator('.studio-tabs').getByRole('button',{name:'Piano',exact:true}).click();
  await expect(page.locator('.studio-page')).toHaveCount(piano.pages.length);
  await page.getByRole('button',{name:'Export PDF',exact:true}).click();
  await expect.poll(()=>page.evaluate(()=>(window as unknown as {studioCalls:Array<{command:string;args:{partId:string;format:string}}>}).studioCalls.find(c=>c.command==='export_score')?.args.format)).toBe('pdf');
  await page.locator('.studio-tabs').getByRole('button',{name:'Full score',exact:true}).click();
  await expect(page.locator('.studio-page')).toHaveCount(full.pages.length);
  await page.locator('.studio-panel').evaluate(element=>element.scrollTop=0);
  await page.screenshot({path:resolve('../../output/pdf/score-studio-canvas.png')});
});
