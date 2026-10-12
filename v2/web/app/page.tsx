'use client';
import {useCallback,useEffect,useState} from 'react';

type User={id:string;reddit_username:string};
type Project={id:string;name:string;description:string;created_at:string};
type Evidence={id:string;source_kind:string;source_id:string;source_url:string;title:string;body:string;collected_at:string};
type Cluster={category:string;independent_stories:number;evidence_count:number;severity_score:number;examples:string[]};
type Report={evidence_scanned:number;independent_stories:number;clusters:Cluster[];caveat:string};
type Run={id:string;topic:string;report:Report;created_at:string};
type Job={id:string;subreddit:string;status:string;attempts:number;last_error:string|null;created_at:string};
type Opportunity={id:string;title:string;problem:string;score:number;evidence_refs:string[];created_at:string};

async function api<T>(path:string,options?:RequestInit):Promise<T>{
 const response=await fetch('/api'+path,{...options,credentials:'same-origin',headers:{'Content-Type':'application/json',...options?.headers}});
 const payload=await response.json().catch(()=>({}));
 if(!response.ok) throw new Error(payload.error||'Request failed');
 return payload as T;
}

export default function Home(){
 const [user,setUser]=useState<User|null>(null);
 const [projects,setProjects]=useState<Project[]>([]);
 const [items,setItems]=useState<Evidence[]>([]);
 const [runs,setRuns]=useState<Run[]>([]);
 const [jobs,setJobs]=useState<Job[]>([]);
 const [opportunities,setOpportunities]=useState<Opportunity[]>([]);
 const [selected,setSelected]=useState('');
 const [subreddit,setSubreddit]=useState('SaaS');
 const [threadId,setThreadId]=useState('');
 const [topic,setTopic]=useState('Manual workflows and recurring customer frustrations');
 const [projectName,setProjectName]=useState('');
 const [busy,setBusy]=useState(false);
 const [notice,setNotice]=useState('');
 const [tab,setTab]=useState<'discover'|'evidence'|'opportunities'|'jobs'>('discover');

 const load=useCallback(async()=>{
   try {
     const me=await api<User>('/me');setUser(me);
     const [p,e,r,j,o]=await Promise.all([
       api<Project[]>('/projects'),api<Evidence[]>('/evidence?limit=200'),api<Run[]>('/research/runs'),
       api<Job[]>('/research/jobs'),api<Opportunity[]>('/opportunities')
     ]);
     setProjects(p);setItems(e);setRuns(r);setJobs(j);setOpportunities(o);
   } catch(e){setUser(null);setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);setNotice(e instanceof Error?e.message:'Unable to load');}
 },[]);
 useEffect(()=>{void load();},[load]);

 async function act(action:()=>Promise<unknown>,message:string){
   setBusy(true);setNotice('');
   try {await action();setNotice(message);await load();}
   catch(e){setNotice(e instanceof Error?e.message:'Operation failed');}
   finally{setBusy(false);}
 }
 const project_id=selected||null;
 const latest=runs[0];
 return <div className="shell">
  <aside className="sidebar">
    <div className="brand"><span className="brand-symbol">◈</span><span>Pain Intelligence<span className="brand-os"> OS</span></span></div>
    <p className="sidebar-caption">Turn conversations into evidence. Turn evidence into decisions.</p>
    <div className="side-section">WORKSPACE</div>
    <nav aria-label="Research navigation">
      {([['discover','◉','Overview'],['evidence','≡','Evidence Library'],['opportunities','◇','Opportunities'],['jobs','↻','Research Jobs']] as const).map(([key,icon,label])=>
        <button key={key} onClick={()=>setTab(key)} className={'side-link '+(tab===key?'active':'')}><span>{icon}</span>{label}</button>)}
    </nav>
    <div className="sidebar-bottom"><span className="status-dot" /> Rust + PostgreSQL
      <p>Authorized Reddit API access only. Research scores are not market forecasts.</p>
    </div>
  </aside>
  <main className="main">
   <header className="topbar"><span>INTELLIGENCE WORKSPACE <span className="sep">/</span> {tab.toUpperCase()}</span>
     <div className="identity">{user?<><span className="avatar">{user.reddit_username.slice(0,1).toUpperCase()}</span><b>u/{user.reddit_username}</b><button onClick={()=>void act(()=>api('/auth/logout',{method:'POST'}).then(()=>{setUser(null);setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);}), 'Disconnected')}>Sign out</button></>:<a className="connect" href="/api/auth/reddit/start">Connect Reddit →</a>}</div>
   </header>
   <section className="intro"><div><div className="eyebrow">RESEARCH SYSTEM / V2</div><h1>Find the problem <em>worth solving.</em></h1>
     <p>Collect authorized customer evidence, uncover recurring frustrations, and build a defensible picture of product opportunities.</p>
   </div><span className="edition">EVIDENCE-FIRST RESEARCH</span></section>
   {notice&&<div className="notice" role="status">{notice}</div>}
   {!user&&<div className="notice">Connect an approved Reddit account to access the private research workspace. API approval and app registration are required.</div>}
   <section className="stats">
     {[['Evidence captured',items.length.toString(),'Independent observations'],['Research runs',runs.length.toString(),'Stored analyses'],['Opportunities',opportunities.length.toString(),'Require human validation'],['Research jobs',jobs.length.toString(),'Queue and history']].map(([label,value,caption])=><div className="stat" key={label}><span>{label}</span><strong>{value}</strong><small>{caption}</small></div>)}
   </section>
   {tab==='discover'&&<div className="two-col">
    <section className="panel"><div className="panel-heading"><span className="eyebrow">01 / DISCOVER</span><h2>Collect evidence</h2><p>Use Reddit OAuth to request a bounded sample of top discussions from an accessible community.</p></div>
     <label htmlFor="project">Research project</label>
     <select id="project" value={selected} onChange={e=>setSelected(e.target.value)}><option value="">All evidence / no project</option>{projects.map(p=><option value={p.id} key={p.id}>{p.name}</option>)}</select>
     <label htmlFor="subreddit">Subreddit</label><div className="prefix-input"><span>r/</span><input id="subreddit" value={subreddit} onChange={e=>setSubreddit(e.target.value)} placeholder="SaaS"/></div>
     <button className="primary" disabled={!user||busy} onClick={()=>void act(()=>api('/research/scan',{method:'POST',body:JSON.stringify({subreddit,project_id,limit:25})}),'Authorized Reddit sample collected')}>{busy?'Processing…':'Collect Reddit sample →'}</button>
     <button className="secondary" disabled={!user||busy} onClick={()=>void act(()=>api('/research/jobs',{method:'POST',body:JSON.stringify({subreddit,project_id})}),'Research job queued')}>Queue a background collection</button>
     <div className="subpanel"><span className="eyebrow">THREAD COMMENTS</span><label htmlFor="thread">Reddit post ID</label><input id="thread" placeholder="e.g. 1abcxyz" value={threadId} onChange={e=>setThreadId(e.target.value)}/><button className="secondary" disabled={!user||busy||!threadId.trim()} onClick={()=>void act(()=>api('/research/thread',{method:'POST',body:JSON.stringify({post_id:threadId,project_id,limit:50})}),'Thread comments collected')}>Collect authorized comments</button></div>
    </section>
    <section className="panel"><div className="panel-heading"><span className="eyebrow">02 / ANALYZE</span><h2>Discover repeating pain</h2><p>Group pain signals using transparent scoring with traceable source references.</p></div>
     <label htmlFor="topic">Research question</label><textarea id="topic" rows={3} value={topic} onChange={e=>setTopic(e.target.value)} />
     <button className="primary" disabled={!user||busy||items.length===0} onClick={()=>void act(()=>api('/research/analyze',{method:'POST',body:JSON.stringify({topic,project_id})}),'New evidence report saved')}>Run evidence analysis →</button>
     <div className="subpanel"><span className="eyebrow">YOUR PROJECTS</span><div className="row"><input aria-label="Project name" placeholder="New research project" value={projectName} onChange={e=>setProjectName(e.target.value)}/><button className="tiny" disabled={!user||busy||!projectName.trim()} onClick={()=>void act(async()=>{await api('/projects',{method:'POST',body:JSON.stringify({name:projectName})});setProjectName('');},'Project created')}>Add</button></div></div>
    </section>
    <section className="panel span-all"><div className="panel-heading"><span className="eyebrow">LATEST ANALYSIS</span><h2>{latest?.topic||'No research report yet'}</h2></div>
    {latest?<><div className="summaryline">{latest.report.evidence_scanned} items scanned · {latest.report.independent_stories} observed source stories</div>
      <div className="cluster-grid">{latest.report.clusters.map(c=><div className="cluster" key={c.category}><div className="cluster-top"><b>{c.category}</b><span>{Math.round(c.severity_score)}/100</span></div><div className="bar"><div style={{width:c.severity_score+'%'}}/></div><small>{c.independent_stories} independent stories · {c.evidence_count} signals</small></div>)}</div><p className="disclaimer">{latest.report.caveat}</p></>:<p className="empty">Collect evidence and run your first analysis to see source-grounded research.</p>}</section>
   </div>}
   {tab==='evidence'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">SOURCE LIBRARY</span><h2>Collected evidence</h2></div>{items.length?<div className="list">{items.map(e=><article key={e.id} className="list-item"><span className="pill">{e.source_kind}</span><h3>{e.title||e.source_id}</h3><p>{e.body.slice(0,260)}</p><a href={e.source_url} target="_blank" rel="noopener noreferrer">View source ↗</a></article>)}</div>:<p className="empty">No evidence collected.</p>}</section>}
   {tab==='opportunities'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">DISCOVERY SHORTLIST</span><h2>Research opportunities</h2><p>Prioritization signals are not proof of willingness to pay.</p></div>{opportunities.length?<div className="list">{opportunities.map(o=><article className="list-item" key={o.id}><div className="score">{Math.round(o.score)} / 100</div><h3>{o.title}</h3><p>{o.problem}</p><small>{o.evidence_refs.length} supporting references</small></article>)}</div>:<p className="empty">Complete an analysis to generate research candidates.</p>}</section>}
   {tab==='jobs'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">AUTOMATION</span><h2>Background collection jobs</h2><p>Approved API access and a valid Reddit session are required.</p></div><button className="secondary" onClick={()=>void load()}>Refresh jobs</button>{jobs.length?<div className="list">{jobs.map(j=><article className="list-item job" key={j.id}><b>r/{j.subreddit}</b><span className="pill">{j.status}</span><small>Attempts: {j.attempts}</small>{j.last_error&&<p>{j.last_error}</p>}</article>)}</div>:<p className="empty">No jobs yet. Queue one from Overview.</p>}</section>}
   <footer>PAIN INTELLIGENCE OS <span>© 2026 · Open-source research infrastructure</span></footer>
  </main>
 </div>;
}
