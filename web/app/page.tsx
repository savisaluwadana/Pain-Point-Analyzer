'use client';
import {useCallback,useEffect,useState} from 'react';

type User={id:string;reddit_username:string;reddit_connected:boolean};
type Project={id:string;name:string;description:string;created_at:string};
type Evidence={id:string;source_kind:string;source_id:string;source_url:string;title:string;body:string;collected_at:string};
type Cluster={category:string;independent_stories:number;evidence_count:number;severity_score:number;examples:string[]};
type Report={evidence_scanned:number;independent_stories:number;clusters:Cluster[];caveat:string};
type Run={id:string;topic:string;report:Report;created_at:string};
type Job={id:string;subreddit:string;status:string;attempts:number;last_error:string|null;created_at:string};
type HostRun={id:string;topic:string;name:string;status:string;clusters:unknown[];opportunities:unknown[];annotations:Record<string,unknown>};
type HostWorkspace={id:string;stage:string;opportunity:{title:string;problem:string};experiments:unknown[]};
type HostJob={id:string;topic:string;status:string;pass:number};
type ApiKey={id:string;label:string;created_at:string;last_used_at:string|null};
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
 const [keys,setKeys]=useState<ApiKey[]>([]);
 const [hostRuns,setHostRuns]=useState<HostRun[]>([]);
 const [hostWorkspaces,setHostWorkspaces]=useState<HostWorkspace[]>([]);
 const [hostJobs,setHostJobs]=useState<HostJob[]>([]);
 const [keyLabel,setKeyLabel]=useState('My MCP client');
 const [newKey,setNewKey]=useState('');
 const [subreddit,setSubreddit]=useState('SaaS');
 const [threadId,setThreadId]=useState('');
 const [manualTitle,setManualTitle]=useState('');
 const [manualBody,setManualBody]=useState('');
 const [manualUrl,setManualUrl]=useState('');
 const [topic,setTopic]=useState('Manual workflows and recurring customer frustrations');
 const [projectName,setProjectName]=useState('');
 const [busy,setBusy]=useState(false);
 const [notice,setNotice]=useState('');
 const [tab,setTab]=useState<'discover'|'evidence'|'opportunities'|'jobs'|'integrations'|'agents'|'settings'>('discover');

 const load=useCallback(async()=>{
   try {
     const me=await api<User>('/me');setUser(me);
     const [p,e,r,j,o,k,hr,hw,hj]=await Promise.all([
       api<Project[]>('/projects'),api<Evidence[]>('/evidence?limit=200'),api<Run[]>('/research/runs'),
       api<Job[]>('/research/jobs'),api<Opportunity[]>('/opportunities'),api<ApiKey[]>('/mcp/keys'),api<HostRun[]>('/mcp/runs'),api<HostWorkspace[]>('/mcp/workspaces'),api<HostJob[]>('/mcp/research-jobs')
     ]);
     setProjects(p);setItems(e);setRuns(r);setJobs(j);setOpportunities(o);setKeys(k);setHostRuns(hr);setHostWorkspaces(hw);setHostJobs(hj);
   } catch(e){setUser(null);setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);setKeys([]);setHostRuns([]);setHostWorkspaces([]);setHostJobs([]);setNotice(e instanceof Error?e.message:'Unable to load');}
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
      {([['discover','◉','Overview'],['evidence','≡','Evidence Library'],['opportunities','◇','Opportunities'],['jobs','↻','Research Jobs'],['agents','♧','Agent Research'],['integrations','⌘','MCP Integrations'],['settings','⚙','Privacy & Settings']] as const).map(([key,icon,label])=>
        <button key={key} onClick={()=>setTab(key)} className={'side-link '+(tab===key?'active':'')}><span>{icon}</span>{label}</button>)}
    </nav>
    <div className="sidebar-bottom"><span className="status-dot" /> Rust + PostgreSQL
      <p>Authorized Reddit API access only. Research scores are not market forecasts.</p>
    </div>
  </aside>
  <main className="main">
   <header className="topbar"><span>INTELLIGENCE WORKSPACE <span className="sep">/</span> {tab.toUpperCase()}</span>
     <div className="identity">{user?<><span className="avatar">{user.reddit_username.slice(0,1).toUpperCase()}</span><b>u/{user.reddit_username}</b>{!user.reddit_connected&&<a className="connect" href="/api/auth/reddit/start">Reconnect Reddit</a>}<button onClick={()=>void act(()=>api('/auth/logout',{method:'POST'}).then(()=>{setUser(null);setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);}), 'Disconnected')}>Sign out</button></>:<a className="connect" href="/api/auth/reddit/start">Connect Reddit →</a>}</div>
   </header>
   <section className="intro"><div><div className="eyebrow">RESEARCH SYSTEM / V2</div><h1>Find the problem <em>worth solving.</em></h1>
     <p>Collect authorized customer evidence, uncover recurring frustrations, and build a defensible picture of product opportunities.</p>
   </div><span className="edition">EVIDENCE-FIRST RESEARCH</span></section>
   {notice&&<div className="notice" role="status">{notice}</div>}
   {!user&&<div className="notice">Connect an approved Reddit account to access the private research workspace. API approval and app registration are required.</div>}
   {user&&!user.reddit_connected&&<div className="notice">Reddit is disconnected. Your existing non-Reddit research remains available; reconnect to collect Reddit posts and comments.</div>}
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
     <div className="subpanel"><span className="eyebrow">YOUR PROJECTS</span><div className="row"><input aria-label="Project name" placeholder="New research project" value={projectName} onChange={e=>setProjectName(e.target.value)}/><button className="tiny" disabled={!user||busy||!projectName.trim()} onClick={()=>void act(async()=>{await api('/projects',{method:'POST',body:JSON.stringify({name:projectName})});setProjectName('');},'Project created')}>Add</button></div><div className="list">{projects.map(project=><div className="list-item job" key={project.id}><b>{project.name}</b><button className="tiny" disabled={busy} onClick={()=>{if(window.confirm('Delete this project? Its evidence and reports remain available as unassigned records.'))void act(async()=>{await api('/projects/'+project.id,{method:'DELETE'});if(selected===project.id)setSelected('');},'Project deleted; research was preserved');}}>Delete</button></div>)}</div></div>
    </section>
    <section className="panel span-all"><div className="panel-heading"><span className="eyebrow">LATEST ANALYSIS</span><h2>{latest?.topic||'No research report yet'}</h2></div>
    {latest?<><div className="summaryline">{latest.report.evidence_scanned} items scanned · {latest.report.independent_stories} observed source stories</div>
      <div className="cluster-grid">{latest.report.clusters.map(c=><div className="cluster" key={c.category}><div className="cluster-top"><b>{c.category}</b><span>{Math.round(c.severity_score)}/100</span></div><div className="bar"><div style={{width:c.severity_score+'%'}}/></div><small>{c.independent_stories} independent stories · {c.evidence_count} signals</small></div>)}</div><p className="disclaimer">{latest.report.caveat}</p></>:<p className="empty">Collect evidence and run your first analysis to see source-grounded research.</p>}</section>
   </div>}
   {tab==='evidence'&&<><section className="panel"><div className="panel-heading"><span className="eyebrow">MANUAL EVIDENCE</span><h2>Add research evidence</h2><p>Add an independently collected source where you have permission to retain the material.</p></div>
     <label htmlFor="manualTitle">Evidence title</label><input id="manualTitle" value={manualTitle} onChange={e=>setManualTitle(e.target.value)} placeholder="Recurring frustration or request"/>
     <label htmlFor="manualUrl">Source URL (optional)</label><input id="manualUrl" value={manualUrl} onChange={e=>setManualUrl(e.target.value)} placeholder="https://example.com/source"/>
     <label htmlFor="manualBody">What did the source say?</label><textarea id="manualBody" rows={4} value={manualBody} onChange={e=>setManualBody(e.target.value)} placeholder="Summarize relevant evidence without unnecessary personal details"/>
     <button className="primary" disabled={!user||busy||!manualTitle.trim()} onClick={()=>void act(async()=>{await api('/evidence',{method:'POST',body:JSON.stringify({source_kind:'manual',source_id:crypto.randomUUID(),source_url:manualUrl,title:manualTitle,body:manualBody,project_id})});setManualTitle('');setManualBody('');setManualUrl('');},'Evidence saved')}>Save evidence →</button>
   </section><section className="panel"><div className="panel-heading"><span className="eyebrow">SOURCE LIBRARY</span><h2>Collected evidence</h2></div>{items.length?<div className="list">{items.map(e=><article key={e.id} className="list-item"><span className="pill">{e.source_kind}</span><h3>{e.title||e.source_id}</h3><p>{e.body.slice(0,260)}</p>{e.source_url&&<a href={e.source_url} target="_blank" rel="noopener noreferrer">View source ↗</a>}<button className="tiny" disabled={busy} onClick={()=>{if(window.confirm('Delete this evidence item? Saved analysis reports are historical snapshots.')) void act(()=>api('/evidence/'+e.id,{method:'DELETE'}),'Evidence deleted');}}>Delete evidence</button></article>)}</div>:<p className="empty">No evidence collected.</p>}</section></>}
   {tab==='opportunities'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">DISCOVERY SHORTLIST</span><h2>Research opportunities</h2><p>Prioritization signals are not proof of willingness to pay.</p></div>{opportunities.length?<div className="list">{opportunities.map(o=><article className="list-item" key={o.id}><div className="score">{Math.round(o.score)} / 100</div><h3>{o.title}</h3><p>{o.problem}</p><small>{o.evidence_refs.length} supporting references</small></article>)}</div>:<p className="empty">Complete an analysis to generate research candidates.</p>}</section>}
   {tab==='jobs'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">AUTOMATION</span><h2>Background collection jobs</h2><p>Approved API access and a valid Reddit session are required.</p></div><button className="secondary" onClick={()=>void load()}>Refresh jobs</button>{jobs.length?<div className="list">{jobs.map(j=><article className="list-item job" key={j.id}><b>r/{j.subreddit}</b><span className="pill">{j.status}</span><small>Attempts: {j.attempts}</small>{j.last_error&&<p>{j.last_error}</p>}</article>)}</div>:<p className="empty">No jobs yet. Queue one from Overview.</p>}</section>}
   {tab==='agents'&&<div className="two-col">
     <section className="panel"><div className="panel-heading"><span className="eyebrow">HOST-POWERED RESEARCH</span><h2>Research jobs</h2><p>Codex or Claude can browse approved sources, supply grounded evidence and ask the Rust API to persist research progress.</p></div>
       <button className="secondary" onClick={()=>void load()}>Refresh agent state</button>
       {hostJobs.length?<div className="list">{hostJobs.map(j=><article className="list-item" key={j.id}><span className="pill">{j.status}</span><h3>{j.topic}</h3><small>Research pass: {j.pass||0}</small></article>)}</div>:<p className="empty">No host-driven jobs yet. Connect an MCP host to start one.</p>}
     </section>
     <section className="panel"><div className="panel-heading"><span className="eyebrow">SEMANTIC RESEARCH</span><h2>LLM-assisted reports</h2><p>Host-provided analysis must reference evidence IDs; the server does not invent or generate source material.</p></div>
       {hostRuns.length?<div className="list">{hostRuns.map(r=><article className="list-item" key={r.id}><span className="pill">{r.status}</span><h3>{r.name||r.topic}</h3><p>{Object.keys(r.annotations||{}).length} annotations · {(r.clusters||[]).length} clusters · {(r.opportunities||[]).length} candidates</p></article>)}</div>:<p className="empty">No model-hosted analysis runs.</p>}
     </section>
     <section className="panel span-all"><div className="panel-heading"><span className="eyebrow">FROM VALIDATION TO EXECUTION</span><h2>Opportunity workspaces</h2><p>Evidence-linked product candidates, validation experiments and execution planning.</p></div>
       {hostWorkspaces.length?<div className="cluster-grid">{hostWorkspaces.map(w=><div className="cluster" key={w.id}><span className="pill">{w.stage}</span><h3>{w.opportunity?.title||'Untitled opportunity'}</h3><p>{w.opportunity?.problem||''}</p><small>{(w.experiments||[]).length} validation experiments</small></div>)}</div>:<p className="empty">Create opportunity workspaces through the MCP research workflow.</p>}
     </section>
   </div>}
   {tab==='integrations'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">NATIVE RUST MCP / HOST INTEGRATION</span><h2>Connect Claude, Codex or another MCP host</h2><p>Create a revocable, per-account key for the local Rust MCP server. Your host controls browsing and model reasoning; source text remains untrusted evidence.</p></div>
    <label htmlFor="mcpKeyLabel">Key description</label><input id="mcpKeyLabel" value={keyLabel} onChange={e=>setKeyLabel(e.target.value)} placeholder="My desktop MCP client"/>
    <button className="primary" disabled={!user||busy||!keyLabel.trim()} onClick={()=>void act(async()=>{const result=await api<{api_key:string}>('/mcp/keys',{method:'POST',body:JSON.stringify({label:keyLabel})});setNewKey(result.api_key);},'API key created. Copy and store it securely now.')}>Generate MCP API key</button>
    {newKey&&<div className="subpanel"><span className="eyebrow">ONE-TIME SECRET — COPY NOW</span><p>Copy this key into your MCP host configuration. It will not be shown again.</p><div className="row"><input aria-label="New API key" type="text" value={newKey} readOnly/><button className="tiny" onClick={()=>void navigator.clipboard.writeText(newKey)}>Copy</button></div></div>}
    <div className="subpanel"><span className="eyebrow">ACTIVE API KEYS</span>{keys.map(k=><div className="list-item job" key={k.id}><b>{k.label}</b><small>{new Date(k.created_at).toLocaleDateString()}</small><button className="tiny" disabled={busy} onClick={()=>void act(async()=>{await api('/mcp/keys/'+k.id,{method:'DELETE'});setNewKey('');},'API key revoked')}>Revoke</button></div>)}</div>
    <div className="subpanel"><span className="eyebrow">RUN THE MCP BRIDGE</span><p>Install Rust and run this command from the repository root with your secret set in <code>PAIN_MCP_API_KEY</code>. Keep API credentials out of committed files.</p>
     <pre className="command">cargo run --manifest-path api/Cargo.toml -- --mcp</pre>
     <p className="disclaimer">Requires a running web/API stack. The bridge exposes 57 original tool schemas. Execution is authenticated and all workflow data stays in your PostgreSQL workspace.</p>
    </div>
   </section>}
   {tab==='settings'&&<section className="panel">
     <div className="panel-heading"><span className="eyebrow">ACCOUNT PRIVACY</span><h2>Your research data and Reddit access</h2><p>These controls operate on your account only. Research and account deletion cannot be undone.</p></div>
     <div className="subpanel">
       <h3>Export your research</h3><p>Download a JSON copy of your projects, evidence, saved analyses and opportunity workspaces. API credentials and Reddit post/comment text are excluded.</p>
       {user?<a className="connect" href="/api/privacy/export" download="pain-intelligence-export.json">Download research JSON ↓</a>:<p>Sign in to download your research.</p>}
     </div>
     <div className="subpanel">
       <h3>Disconnect Reddit</h3><p>Revoke this app's Reddit OAuth token, stop pending Reddit collections and delete stored Reddit posts and comments. Other projects, research summaries and MCP keys remain. To erase those too, delete the account.</p>
       <button className="secondary" disabled={!user||!user.reddit_connected||busy} onClick={()=>{
         if(window.confirm('Revoke Reddit access and delete stored Reddit post/comment evidence?'))
           void act(()=>api('/auth/reddit/disconnect',{method:'POST'}),'Reddit disconnected and source evidence removed');
       }}>Disconnect and revoke Reddit access</button>
     </div>
     <div className="subpanel">
       <h3>Delete account and all stored data</h3><p>Delete this account, sessions, projects, evidence, research reports, opportunity workspaces and API keys. The service also attempts to revoke Reddit's token before deletion. If that fails, revoke the app manually in Reddit account settings.</p>
       <button className="secondary" disabled={!user||busy} onClick={()=>{
         if(window.confirm('Permanently delete your account and ALL its stored research data? This cannot be undone.'))
           void act(()=>api('/privacy/delete-data',{method:'POST'}),'Your stored account data was deleted');
       }}>Delete my account and data</button>
     </div>
   </section>}
   <footer>PAIN INTELLIGENCE OS <span>© 2026 · Open-source research infrastructure</span></footer>
  </main>
 </div>;
}
