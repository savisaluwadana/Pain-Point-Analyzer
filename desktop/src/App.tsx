import { useCallback, useEffect, useMemo, useState, type FormEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Decision, Opportunity, Workspace } from './types';

type View = 'overview' | 'evidence' | 'analysis' | 'opportunities' | 'settings';
const empty: Workspace = { evidence: [], runs: [], opportunities: [] };
const decisions: { value: Decision; label: string }[] = [
  { value:'unreviewed', label:'Not reviewed' },
  { value:'investigating', label:'Investigating' },
  { value:'needs_validation', label:'Needs validation' },
  { value:'ready_to_build', label:'Ready to build (manual decision)' },
  { value:'not_pursuing', label:'Not pursuing' },
];
const tabs: { view:View; name:string; glyph:string }[] = [
  { view:'overview', name:'Overview', glyph:'▦' },
  { view:'evidence', name:'Evidence library', glyph:'☷' },
  { view:'analysis', name:'Analyze & discover', glyph:'◎' },
  { view:'opportunities', name:'Project ideas', glyph:'◇' },
  { view:'settings', name:'Storage & privacy', glyph:'⚙' },
];
function humanDate(raw:string){try {return new Date(raw).toLocaleString();}catch{return raw;}}
function sourceLink(raw:string){
  try { const u = new URL(raw); return ['http:','https:'].includes(u.protocol) ? u.href : undefined; }
  catch { return undefined; }
}
function TextField({label,value,onChange,rows,placeholder}: {label:string;value:string;onChange:(value:string)=>void;rows?:number;placeholder?:string}) {
  return <label className="field"><span>{label}</span>{rows ? <textarea rows={rows} value={value} placeholder={placeholder} onChange={e=>onChange(e.target.value)}/>
    : <input value={value} placeholder={placeholder} onChange={e=>onChange(e.target.value)}/>}</label>;
}
function Review({item,snapshot,refresh,mutate}:{item:Opportunity;snapshot:Workspace;refresh:()=>Promise<void>;mutate:<T,>(action:()=>Promise<T>,success:string)=>Promise<{ok:true;value:T}|{ok:false}>}) {
  const [decision,setDecision]=useState<Decision>(item.decision);
  const [notes,setNotes]=useState(item.validation_notes);
  const [solutions,setSolutions]=useState(item.existing_solutions);
  const [expanded,setExpanded]=useState(false);
  const supported = useMemo(()=>snapshot.evidence.filter(e=>item.evidence_refs.includes(e.id)),[item.evidence_refs,snapshot.evidence]);
  const missing = item.evidence_refs.length-supported.length;
  return <article className="opportunity">
    <div className="op-head"><div><span className="eyebrow">{item.independent_stories} independent {item.independent_stories===1?'discussion':'discussions'}</span><h3>{item.title}</h3></div><span className="score">Relative priority {Math.round(item.score)}/100</span></div>
    <p className="muted">{item.evidence_refs.length} supporting records · Current decision: {decisions.find(d=>d.value===item.decision)?.label}</p>
    <button className="text-button" aria-expanded={expanded} onClick={()=>setExpanded(v=>!v)}>{expanded?'Hide review':'Inspect evidence & review'} <span aria-hidden="true">{expanded?'−':'+'}</span></button>
    {expanded && <div className="review-panel">
      <p className="info-note">This is a rule-based hypothesis, not proven customer demand. Source evidence is shown from the <strong>current</strong> local database. Removed items are unavailable.</p>
      {supported.length ? <div className="evidence-list">{supported.map(e=><div className="evidence-mini" key={e.id}><strong>{e.title}</strong><p>{e.body.slice(0,350)}</p>{sourceLink(e.source_url)&&<small className="url-text">Source: {e.source_url}</small>}</div>)}</div> : <p className="muted">No currently available supporting evidence.</p>}
      {missing>0&&<p className="warning">{missing} saved reference(s) were deleted, expired or aren't in the latest loaded records. Original evidence is not embedded in saved reports.</p>}
      <TextField label="Existing solutions / workarounds" rows={3} value={solutions} onChange={setSolutions} placeholder="Notes on alternative projects and existing approaches"/>
      <TextField label="Validation notes" rows={4} value={notes} onChange={setNotes} placeholder="Your interviews, contradictory evidence and open questions"/>
      <label className="field"><span>Manual decision</span><select value={decision} onChange={e=>setDecision(e.target.value as Decision)}>{decisions.map(d=><option value={d.value} key={d.value}>{d.label}</option>)}</select></label>
      <button className="primary" disabled={decision==='ready_to_build'&&notes.trim().length<20} onClick={async()=>{const saved=await mutate(()=>invoke<void>('save_review',{id:item.id,decision,notes,solutions}),'Review saved');if(saved.ok)await refresh();}}>Save review</button>
    </div>}
  </article>;
}
export default function App(){
  const [view,setView]=useState<View>('overview');
  const [snapshot,setSnapshot]=useState<Workspace>(empty);
  const [loading,setLoading]=useState(true);
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState('');
  const [notice,setNotice]=useState('');
  const [title,setTitle]=useState('');
  const [body,setBody]=useState('');
  const [url,setUrl]=useState('');
  const [topic,setTopic]=useState('Synthetic demo: ways open-source projects could help communities');
  const [exportPath,setExportPath]=useState('');
  const refresh=useCallback(async()=>{const state=await invoke<Workspace>('load_workspace');setSnapshot(state);},[]);
  useEffect(()=>{refresh().catch(e=>setError(String(e))).finally(()=>setLoading(false));},[refresh]);
  const mutate=useCallback(async <T,>(action:()=>Promise<T>,success:string):Promise<{ok:true;value:T}|{ok:false}>=>{
    setBusy(true);setError('');setNotice('');try{const outcome=await action();setNotice(success);return {ok:true,value:outcome};}catch(e){setError(String(e));return {ok:false};}finally{setBusy(false);}
  },[]);
  const latest=snapshot.runs[0];
  const freshIdeas=snapshot.opportunities.filter(o=>o.run_id===latest?.id);
  async function save(e:FormEvent){e.preventDefault();if(!title.trim()||!body.trim())return;
    const result=await mutate(()=>invoke<void>('save_evidence',{entry:{title,body,source_url:url}}),'Evidence saved locally');
    if(result.ok){setTitle('');setBody('');setUrl('');await refresh();}
  }
  async function addDemo(){const result=await mutate(()=>invoke<number>('seed_demo'),'Fictional example records added');if(result.ok)await refresh();}
  async function analyze(){const result=await mutate(()=>invoke<void>('run_analysis',{topic}),'Analysis complete and saved');if(result.ok){await refresh();setView('opportunities');}}
  return <div className="shell">
    <aside className="sidebar" aria-label="Main navigation">
      <div className="brand-mark">◈ <span>Discovery<span className="light">Lab</span></span></div>
      <p className="brand-caption">Community research, made local.</p>
      <span className="nav-label">WORKSPACE</span>
      <nav aria-label="Desktop sections">{tabs.map(tab=><button type="button" key={tab.view} className={'nav-item '+(view===tab.view?'active':'')} aria-current={view===tab.view?'page':undefined} onClick={()=>setView(tab.view)}><span aria-hidden="true" className="nav-glyph">{tab.glyph}</span>{tab.name}</button>)}</nav>
      <div className="sidebar-footer"><span className="status-dot"/> Local desktop edition <small>Rust · SQLite · No server needed</small></div>
    </aside>
    <main className="content">
      <header className="topbar"><div><span className="eyebrow">OPEN SOURCE DISCOVERY LAB / DESKTOP ALPHA</span><h1>{tabs.find(t=>t.view===view)?.name}</h1></div><span className="local-pill">● Data stored on this device</span></header>
      {error&&<div className="message error" role="alert">{error}<button onClick={()=>setError('')} aria-label="Dismiss error">×</button></div>}
      {notice&&<div className="message success" role="status">{notice}<button onClick={()=>setNotice('')} aria-label="Dismiss notice">×</button></div>}
      {loading?<section className="card">Loading your local research database…</section>:<>
      {view==='overview'&&<>
        <section className="intro-card"><span className="eyebrow">THE DESKTOP WORKSPACE</span><h2>Find patterns in problems.<br/><em>Build something useful.</em></h2><p>Capture permitted research evidence, identify recurring themes using the same Rust analyzer as the web edition, and record your own validation decisions. Your data stays on this computer.</p><div className="intro-actions"><button className="primary" disabled={busy} onClick={()=>setView('evidence')}>Add evidence</button><button className="secondary" disabled={busy} onClick={addDemo}>Load fictional demo</button></div></section>
        <section className="metrics"><div><span>Evidence entries</span><strong>{snapshot.evidence.length}</strong><small>Most recent 500</small></div><div><span>Saved research runs</span><strong>{snapshot.runs.length}</strong><small>Most recent 100</small></div><div><span>Potential themes</span><strong>{snapshot.opportunities.length}</strong><small>Need human validation</small></div></section>
        <section className="card"><div className="card-head"><div><span className="eyebrow">GET STARTED</span><h2>A simple research loop</h2></div></div><div className="steps"><div><b>01</b><strong>Collect</strong><p>Write short, permitted summaries with a source reference.</p></div><div><b>02</b><strong>Analyze</strong><p>Group keyword-matched evidence into research themes.</p></div><div><b>03</b><strong>Review</strong><p>Inspect original context, record alternative solutions, and validate manually.</p></div></div></section>
        <p className="fineprint">This desktop alpha currently supports local manual evidence and fictional demonstration data. Remote collectors and MCP integration remain in the web edition while desktop parity is developed.</p>
      </>}
      {view==='evidence'&&<div className="stack"><section className="card"><div className="card-head"><div><span className="eyebrow">MANUAL EVIDENCE</span><h2>Record a problem worth investigating</h2><p>Use your own short summary of a permitted source. Do not paste personal information or Reddit content.</p></div></div><form className="entry-form" onSubmit={save}><TextField label="Evidence title" value={title} onChange={setTitle} placeholder="e.g. Maintainers repeatedly export spreadsheets"/><TextField label="Source URL (optional)" value={url} onChange={setUrl} placeholder="https://example.com/thread"/><TextField label="Research observation" rows={5} value={body} onChange={setBody} placeholder="Summarize the problem, workaround and context in your own words"/><div><button className="primary" disabled={busy||!title.trim()||!body.trim()} type="submit">Save to this computer</button> <button className="secondary" type="button" disabled={busy} onClick={addDemo}>Add six fictional examples</button></div></form></section>
        <section className="card"><div className="card-head"><div><span className="eyebrow">YOUR EVIDENCE</span><h2>Saved observations</h2></div><span className="count">{snapshot.evidence.length} shown</span></div>{snapshot.evidence.length?<div className="evidence-list">{snapshot.evidence.map(e=><article className="evidence-entry" key={e.id}><div className="entry-line"><span className="tag">{e.source_kind==='demo'?'FICTIONAL DEMO':'MANUAL'}</span><time>{humanDate(e.created_at)}</time></div><h3>{e.title}</h3><p>{e.body}</p>{sourceLink(e.source_url)&&<small className="url-text">Source: {e.source_url}</small>}<button className="danger-link" disabled={busy} onClick={async()=>{if(window.confirm('Permanently delete this evidence from the local library? Historical report counts will remain.')){const result=await mutate(()=>invoke<void>('remove_evidence',{id:e.id}),'Evidence removed');if(result.ok)await refresh();}}}>Delete entry</button></article>)}</div>:<p className="empty">No evidence yet. Add one or use the fictional example.</p>}</section></div>}
      {view==='analysis'&&<div className="stack"><section className="card"><span className="eyebrow">KEYWORDS-V2 / SHARED RUST ENGINE</span><h2>Look for recurring pain points</h2><p>Research runs scan up to 1,000 recent local evidence records. The keyword engine is deterministic and cannot infer buyer intent or understand all paraphrases.</p><TextField label="Research question" rows={3} value={topic} onChange={setTopic}/><div className="form-actions"><button className="primary" disabled={busy||!snapshot.evidence.length||!topic.trim()} onClick={analyze}>{busy?'Working…':'Run local analysis'}</button><span className="fineprint">No AI API key · No outgoing request</span></div></section>
      {latest?<section className="card"><span className="eyebrow">LATEST RESEARCH / {humanDate(latest.created_at)}</span><h2>{latest.topic}</h2><div className="analysis-metrics"><span>{latest.report.evidence_scanned} scanned</span><span>{latest.report.independent_stories} independent stories</span><span>{latest.report.clusters.length} themes</span><span>{latest.report.unclassified_items} unclassified</span></div><p className="fineprint">{latest.report.caveat}</p><div className="evidence-list">{latest.report.clusters.map(c=><div key={c.category} className="result-line"><strong>{c.category}</strong><span>{c.independent_stories} discussions</span><span>relative {Math.round(c.severity_score)}/100</span>{!c.sufficient_evidence&&<span className="warning">Sparse</span>}</div>)}</div></section>:<section className="card empty">No saved analysis. Add evidence and run your first analysis.</section>}</div>}
      {view==='opportunities'&&<section className="card"><div className="card-head"><div><span className="eyebrow">SOURCE-GROUNDED CANDIDATES</span><h2>Project opportunities</h2><p>Shown for the latest research run. Scores are relative to that run, not verified demand.</p></div></div>{freshIdeas.length?<div className="stack">{freshIdeas.map(i=><Review key={i.id} item={i} snapshot={snapshot} refresh={refresh} mutate={mutate}/>)}</div>:<div className="empty">No ideas yet. Run an evidence analysis to generate candidate themes.</div>}</section>}
      {view==='settings'&&<div className="stack"><section className="card"><span className="eyebrow">LOCAL BY DESIGN</span><h2>Privacy and storage</h2><p>The app saves research to SQLite in your operating system's per-user application data directory. No PostgreSQL container, Node server or HTTP port is running in production desktop mode.</p><p>This first desktop alpha runs under your device user account. Lock your device and protect your OS profile to restrict access to local research. The desktop edition has no remote source collectors, auth accounts or automatic sync yet.</p><button className="primary" disabled={busy} onClick={async()=>{const result=await mutate(()=>invoke<string>('export_workspace'),'JSON snapshot written locally');if(result.ok)setExportPath(result.value);}}>Export research as JSON</button>{exportPath&&<p className="saved-path">Saved to: <span>{exportPath}</span></p>}<p className="fineprint">Exports contain your entered evidence and notes. Store them securely. JSON export is not yet an automated restore/import mechanism.</p></section><section className="card"><h2>About the desktop alpha</h2><p><strong>Rust + SQLite + React + Tauri 2</strong>. The Rust keywords-v2 analyzer is shared with the existing Docker/web implementation. Provider-specific collection, PostgreSQL migration/import, auto-update, signed releases and full MCP are future phases.</p></section></div>}
      </>}
      <footer>OPEN SOURCE DISCOVERY LAB <span>Local alpha · Individual, currently non-commercial research · Original software MIT-licensed</span></footer>
    </main>
  </div>;
}
