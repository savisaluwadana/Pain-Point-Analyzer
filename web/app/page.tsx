'use client';
import {useCallback,useEffect,useState} from 'react';

type User={id:string;reddit_username:string;reddit_connected:boolean;local_account?:boolean};
type Project={id:string;name:string;description:string;created_at:string};
type Evidence={id:string;source_kind:string;source_id:string;source_url:string;title:string;body:string;metadata?:{details?:{author?:string;license?:string;license_url?:string}};collected_at:string};
type Cluster={category:string;independent_stories:number;evidence_count:number;severity_score:number;examples:string[]};
type Report={evidence_scanned:number;independent_stories:number;clusters:Cluster[];caveat:string};
type Run={id:string;project_id:string|null;topic:string;report:Report;created_at:string};
type Job={id:string;subreddit:string;status:string;attempts:number;last_error:string|null;created_at:string};
type HostRun={id:string;topic:string;name:string;status:string;clusters:unknown[];opportunities:unknown[];annotations:Record<string,unknown>};
type HostWorkspace={id:string;stage:string;opportunity:{title:string;problem:string};experiments:unknown[]};
type HostJob={id:string;topic:string;status:string;pass:number};
type ApiKey={id:string;label:string;created_at:string;last_used_at:string|null};
type Opportunity={id:string;title:string;problem:string;score:number;evidence_refs:string[];created_at:string};
type SourceOption={id:string;name:string;mode:'keyword'|'video'|'forum'|'tag'|'subreddit';ready:boolean;note:string};
type SourceCatalog={sources:SourceOption[]};


async function api<T>(path:string,options?:RequestInit):Promise<T>{
 const response=await fetch('/api'+path,{...options,credentials:'same-origin',headers:{'Content-Type':'application/json',...options?.headers}});
 const payload=await response.json().catch(()=>({}));
 if(!response.ok) throw new Error(payload.error||'Request failed');
 return payload as T;
}

// A single optional endpoint failure must not invalidate an otherwise valid login.
async function loadList<T>(request:Promise<T[]>,apply:(items:T[])=>void):Promise<boolean>{
 try { apply(await request); return true; }
 catch { apply([]); return false; }
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
 const [subreddit,setSubreddit]=useState('opensource');
 const [localToken,setLocalToken]=useState('');
 const [catalog,setCatalog]=useState<SourceOption[]>([]);
 const [source,setSource]=useState('github');
 const [sourceQuery,setSourceQuery]=useState('feature request');
 const [sourceTarget,setSourceTarget]=useState('');
 const [sourceResult,setSourceResult]=useState('');
 const [archiveRanking,setArchiveRanking]=useState('top');
 const [archiveDays,setArchiveDays]=useState(30);
 const [archivePost,setArchivePost]=useState('');

 const [threadId,setThreadId]=useState('');
 const [manualTitle,setManualTitle]=useState('');
 const [manualBody,setManualBody]=useState('');
 const [manualUrl,setManualUrl]=useState('');
 const [topic,setTopic]=useState('Recurring community needs that open-source projects could address');
 const [projectName,setProjectName]=useState('');
 const [busy,setBusy]=useState(false);
 const [notice,setNotice]=useState('');
 const [tab,setTab]=useState<'discover'|'sources'|'evidence'|'opportunities'|'jobs'|'integrations'|'agents'|'settings'>('sources');

 const load=useCallback(async()=>{
   try {
     const me=await api<User>('/me');
     setUser(me);
   } catch(e){
     setUser(null);
     setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);
     setKeys([]);setHostRuns([]);setHostWorkspaces([]);setHostJobs([]);
     setNotice(e instanceof Error?e.message:'Unable to authenticate');
     return;
   }
   // Keep the authenticated session visible when a separate research endpoint is unavailable.
   const results=await Promise.all([
     loadList(api<Project[]>('/projects'),setProjects),
     loadList(api<Evidence[]>('/evidence?limit=200'),setItems),
     loadList(api<Run[]>('/research/runs'),setRuns),
     loadList(api<Job[]>('/research/jobs'),setJobs),
     loadList(api<Opportunity[]>('/opportunities'),setOpportunities),
     loadList(api<ApiKey[]>('/mcp/keys'),setKeys),
     loadList(api<HostRun[]>('/mcp/runs'),setHostRuns),
     loadList(api<HostWorkspace[]>('/mcp/workspaces'),setHostWorkspaces),
     loadList(api<HostJob[]>('/mcp/research-jobs'),setHostJobs)
   ]);
   if(results.some(success=>!success))setNotice('Some research sections could not load. Your session is still active; refresh to retry.');
 },[]);
 useEffect(()=>{void load();},[load]);
 useEffect(()=>{void api<SourceCatalog>('/sources').then(v=>setCatalog(v.sources)).catch(()=>setCatalog([]));},[]);
 const chosenSource=catalog.find(x=>x.id===source);


 async function act(action:()=>Promise<unknown>,message:string){
   setBusy(true);setNotice('');
   try {await action();setNotice(message);await load();}
   catch(e){setNotice(e instanceof Error?e.message:'Operation failed');}
   finally{setBusy(false);}
 }
 const project_id=selected||null;
 const latest=runs.find(r=>!selected||r.project_id===selected);
 return <div className="shell">
  <aside className="sidebar">
    <div className="brand"><span className="brand-symbol">◈</span><span>Open Source<span className="brand-os"> Discovery Lab</span></span></div>
    <p className="sidebar-caption">From community conversations to useful open-source contributions.</p>
    <div className="side-section">WORKSPACE</div>
    <nav aria-label="Research navigation">
      {([['sources','⌁','Community Sources'],['discover','◉','Analyze & Discover'],['evidence','≡','Evidence Library'],['opportunities','◇','Project Ideas'],['jobs','↻','Research Jobs'],['agents','♧','Assisted Research'],['integrations','⌘','MCP Integrations'],['settings','⚙','Privacy & Settings']] as const).map(([key,icon,label])=>
        <button key={key} onClick={()=>setTab(key)} className={'side-link '+(tab===key?'active':'')}><span>{icon}</span>{label}</button>)}
    </nav>
    <div className="sidebar-bottom"><span className="status-dot" /> Rust + PostgreSQL
      <p>Public APIs and approved sources only. Research themes require community validation.</p>
    </div>
  </aside>
  <main className="main">
   <header className="topbar"><span>OPEN-SOURCE RESEARCH WORKSPACE <span className="sep">/</span> {tab.toUpperCase()}</span>
     <div className="identity">{user?<><span className="avatar">{user.reddit_username.slice(0,1).toUpperCase()}</span><b>{user.local_account?'Local workspace':'u/'+user.reddit_username}</b>{!user.reddit_connected&&!user.local_account&&<a className="connect" href="/api/auth/reddit/start">Reconnect Reddit</a>}<button onClick={()=>void act(()=>api('/auth/logout',{method:'POST'}).then(()=>{setUser(null);setProjects([]);setItems([]);setRuns([]);setJobs([]);setOpportunities([]);}), 'Disconnected')}>Sign out</button></>:<a className="connect" href="/api/auth/reddit/start">Connect Reddit (optional) →</a>}</div>
   </header>
   <section className="intro"><div><div className="eyebrow">COMMUNITY-FIRST DISCOVERY</div><h1>Discover needs. <em>Build in the open.</em></h1>
     <p>Explore permitted community discussions, find recurring needs, and shape ideas for useful open-source projects. Currently intended for individual, non-commercial research.</p>
   </div><span className="edition">OPEN-SOURCE PROJECT DISCOVERY</span></section>
   {notice&&<div className="notice" role="status">{notice}</div>}
   {!user&&<section className="panel"><div className="panel-heading"><span className="eyebrow">LOCAL RESEARCH ACCESS</span><h2>Sign in without Reddit</h2><p>If LOCAL_LOGIN_TOKEN is configured, enter your locally generated token. No Reddit API approval is needed for other permitted sources.</p></div><label htmlFor="localToken">Local access token</label><input id="localToken" type="password" autoComplete="off" value={localToken} onChange={e=>setLocalToken(e.target.value)} placeholder="Paste the token from your local .env"/><button className="primary" disabled={busy||!localToken.trim()} onClick={()=>void act(async()=>{await api('/auth/local',{method:'POST',body:JSON.stringify({token:localToken})});setLocalToken('');},'Local research workspace connected')}>Sign in locally →</button><p className="disclaimer">Alternatively, connect an approved Reddit account using the header link.</p></section>}
   {user&&!user.reddit_connected&&!user.local_account&&<div className="notice">Reddit is disconnected. Your existing non-Reddit research remains available.</div>}
   <section className="stats">
     {[['Evidence displayed',items.length.toString(),'Latest 200 records loaded'],['Research reviews',runs.length.toString(),'Latest reports loaded'],['Open-source ideas',opportunities.length.toString(),'Require community validation'],['Reddit jobs',jobs.length.toString(),'Queue and history']].map(([label,value,caption])=><div className="stat" key={label}><span>{label}</span><strong>{value}</strong><small>{caption}</small></div>)}
   </section>
   {tab==='discover'&&<div className="two-col">
    <section className="panel"><div className="panel-heading"><span className="eyebrow">01 / DISCOVER</span><h2>Collect evidence</h2><p>With approved Reddit OAuth, collect a small read-only sample of community discussions for local research.</p></div>
     <label htmlFor="project">Research project</label>
     <select id="project" value={selected} onChange={e=>setSelected(e.target.value)}><option value="">All evidence / no project</option>{projects.map(p=><option value={p.id} key={p.id}>{p.name}</option>)}</select>
     <label htmlFor="subreddit">Subreddit</label><div className="prefix-input"><span>r/</span><input id="subreddit" value={subreddit} onChange={e=>setSubreddit(e.target.value)} placeholder="opensource"/></div>
     <button className="primary" disabled={!user||busy} onClick={()=>void act(()=>api('/research/scan',{method:'POST',body:JSON.stringify({subreddit,project_id,limit:25})}),'Authorized Reddit sample collected')}>{busy?'Processing…':'Collect Reddit sample →'}</button>
     <button className="secondary" disabled={!user||busy} onClick={()=>void act(()=>api('/research/jobs',{method:'POST',body:JSON.stringify({subreddit,project_id})}),'Research job queued')}>Queue a background collection</button>
     <div className="subpanel"><span className="eyebrow">THREAD COMMENTS</span><label htmlFor="thread">Reddit post ID</label><input id="thread" placeholder="e.g. 1abcxyz" value={threadId} onChange={e=>setThreadId(e.target.value)}/><button className="secondary" disabled={!user||busy||!threadId.trim()} onClick={()=>void act(()=>api('/research/thread',{method:'POST',body:JSON.stringify({post_id:threadId,project_id,limit:50})}),'Thread comments collected')}>Collect authorized comments</button></div>
    </section>
    <section className="panel"><div className="panel-heading"><span className="eyebrow">02 / ANALYZE</span><h2>Discover recurring themes</h2><p>Group recurring challenges and feature requests into evidence-linked themes. Review findings with communities before proposing a project.</p></div>
     <label htmlFor="topic">Research question</label><textarea id="topic" rows={3} value={topic} onChange={e=>setTopic(e.target.value)} />
     <button className="primary" disabled={!user||busy||items.length===0} onClick={()=>void act(()=>api('/research/analyze',{method:'POST',body:JSON.stringify({topic,project_id})}),'Community research review saved')}>Run evidence analysis →</button>
     <div className="subpanel"><span className="eyebrow">YOUR PROJECTS</span><div className="row"><input aria-label="Project name" placeholder="New research project" value={projectName} onChange={e=>setProjectName(e.target.value)}/><button className="tiny" disabled={!user||busy||!projectName.trim()} onClick={()=>void act(async()=>{await api('/projects',{method:'POST',body:JSON.stringify({name:projectName})});setProjectName('');},'Project created')}>Add</button></div><div className="list">{projects.map(project=><div className="list-item job" key={project.id}><b>{project.name}</b><button className="tiny" disabled={busy} onClick={()=>{if(window.confirm('Delete this project? Its evidence and reports remain available as unassigned records.'))void act(async()=>{await api('/projects/'+project.id,{method:'DELETE'});if(selected===project.id)setSelected('');},'Project deleted; research was preserved');}}>Delete</button></div>)}</div></div>
    </section>
    <section className="panel span-all"><div className="panel-heading"><span className="eyebrow">LATEST ANALYSIS</span><h2>{latest?.topic||'No research report yet'}</h2></div>
    {latest?<><div className="summaryline">{latest.report.evidence_scanned} items scanned · {latest.report.independent_stories} observed source stories</div>
      <div className="cluster-grid">{latest.report.clusters.map(c=><div className="cluster" key={c.category}><div className="cluster-top"><b>{c.category}</b><span>{Math.round(c.severity_score)}/100</span></div><div className="bar"><div style={{width:c.severity_score+'%'}}/></div><small>{c.independent_stories} independent stories · {c.evidence_count} signals</small></div>)}</div><p className="disclaimer">{latest.report.caveat}</p></>:<p className="empty">Collect evidence and run your first analysis to see source-grounded community research.</p>}</section>
   </div>}
   {tab==='sources'&&<div className="two-col">
     <section className="panel"><div className="panel-heading"><span className="eyebrow">READ-ONLY COMMUNITY CONNECTORS</span><h2>Collect source-linked evidence</h2><p>Alpha preview: start with GitHub or try the <a href="https://github.com/savisaluwadana/Reddit-post-Analyzer/blob/main/docs/DEMO.md" target="_blank" rel="noopener noreferrer">offline sample walkthrough ↗</a>. Sources have quotas and may refuse access. No Reddit API credentials required.</p></div>
      <label htmlFor="sourceSelect">Source</label><select id="sourceSelect" value={source} onChange={e=>{setSource(e.target.value);setSourceResult('');if(e.target.value==='medium')setSourceQuery('open-source');else if(e.target.value==='youtube'||e.target.value==='arctic-shift')setSourceQuery('');else setSourceQuery('feature request');setSourceTarget('');}}>{catalog.map(s=><option key={s.id} value={s.id}>{s.name}{s.ready?'':' (needs configuration)'}</option>)}</select>
      {chosenSource&&<p className="disclaimer">{chosenSource.note}</p>}
      <label htmlFor="sourceQuery">{chosenSource?.mode==='subreddit'?'Optional phrase filter within the sample':chosenSource?.mode==='tag'?'Medium topic tag':chosenSource?.mode==='video'?'Optional comment search filter':'Search topic or phrase'}</label>
      <input id="sourceQuery" value={sourceQuery} onChange={e=>setSourceQuery(e.target.value)} placeholder={chosenSource?.mode==='tag'?'open-source':'missing feature'}/>
      {(chosenSource?.mode==='video'||chosenSource?.mode==='forum'||chosenSource?.mode==='subreddit')&&<><label htmlFor="sourceTarget">{chosenSource.mode==='subreddit'?'Subreddit (without r/)':chosenSource.mode==='video'?'YouTube video ID':'Allowlisted Discourse hostname'}</label><input id="sourceTarget" value={sourceTarget} onChange={e=>setSourceTarget(e.target.value)} placeholder={chosenSource.mode==='subreddit'?'freelance':chosenSource.mode==='video'?'dQw4w9WgXcQ':'meta.discourse.org'}/></>}
      {source==='arctic-shift'&&<>
       <label htmlFor="archiveRanking">Selection within the archive sample</label><select id="archiveRanking" value={archiveRanking} onChange={e=>setArchiveRanking(e.target.value)}><option value="top">Highest archived scores</option><option value="mid">Middle of score ranking</option><option value="new">Newest</option></select>
       <label htmlFor="archiveDays">Look back (days)</label><input id="archiveDays" type="number" min={1} max={365} value={archiveDays} onChange={e=>setArchiveDays(Number(e.target.value))}/>
       <label htmlFor="archivePost">Optional post ID for comments</label><input id="archivePost" value={archivePost} onChange={e=>setArchivePost(e.target.value)} placeholder="Leave blank to collect posts"/>
       <p className="disclaimer">Selects up to 10 items from at most 100 recent archive records. Scores may be stale. Wait at least 10 seconds between calls. Use no more than 10 communities per research run.</p>
      </>}
      <label htmlFor="sourceProject">Research project</label><select id="sourceProject" value={selected} onChange={e=>setSelected(e.target.value)}><option value="">Unassigned evidence</option>{projects.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select>
      <button className="primary" disabled={!user||busy||!chosenSource||!chosenSource.ready} onClick={()=>{setSourceResult('');void act(async()=>{const result=await api<{saved:number;fetched?:number;eligible?:number;note?:string}>('/sources/collect',{method:'POST',body:JSON.stringify({source,query:sourceQuery,target:sourceTarget,project_id,limit:source==='arctic-shift'?10:15,...(source==='arctic-shift'?{days:archiveDays,ranking:archiveRanking,sample_size:100,post_id:archivePost.trim()||undefined}:{})})});setSourceResult(result.saved+' new or updated evidence records saved'+(result.fetched!==undefined?' from '+result.fetched+' fetched / '+result.eligible+' eligible. '+result.note:''));},'Source collection completed');}}>{busy?'Collecting…':'Collect source evidence →'}</button>
      {sourceResult&&<p className="disclaimer" role="status">{sourceResult}</p>}
     </section>
     <section className="panel"><div className="panel-heading"><span className="eyebrow">CONNECTOR STATUS</span><h2>Supported sources</h2></div><div className="list">{catalog.map(s=><div key={s.id} className="list-item"><b>{s.name}</b><span className="pill">{s.ready?'Available to try':'Credentials / allowlist needed'}</span><p>{s.note}</p></div>)}</div><p className="disclaimer">Content rights, API quotas, and provider policies still apply. Confirm original context before building an open-source project.</p></section>
    </div>}
    {tab==='evidence'&&<><section className="panel"><div className="panel-heading"><span className="eyebrow">MANUAL EVIDENCE</span><h2>Add research evidence</h2><p>Add an independently collected source where you have permission to retain the material.</p></div>
     <label htmlFor="manualTitle">Evidence title</label><input id="manualTitle" value={manualTitle} onChange={e=>setManualTitle(e.target.value)} placeholder="Community request or recurring challenge"/>
     <label htmlFor="manualUrl">Source URL (optional)</label><input id="manualUrl" value={manualUrl} onChange={e=>setManualUrl(e.target.value)} placeholder="https://example.com/source"/>
     <label htmlFor="manualBody">What did the source say?</label><textarea id="manualBody" rows={4} value={manualBody} onChange={e=>setManualBody(e.target.value)} placeholder="Summarize relevant evidence without unnecessary personal details"/>
     <button className="primary" disabled={!user||busy||!manualTitle.trim()} onClick={()=>void act(async()=>{await api('/evidence',{method:'POST',body:JSON.stringify({source_kind:'manual',source_id:crypto.randomUUID(),source_url:manualUrl,title:manualTitle,body:manualBody,project_id})});setManualTitle('');setManualBody('');setManualUrl('');},'Evidence saved')}>Save evidence →</button>
   </section><section className="panel"><div className="panel-heading"><span className="eyebrow">SOURCE LIBRARY</span><h2>Collected evidence</h2></div>{items.length?<div className="list">{items.map(e=><article key={e.id} className="list-item"><span className="pill">{e.source_kind}</span><h3>{e.title||e.source_id}</h3><p>{e.body.slice(0,260)}</p>{e.source_kind==='stackoverflow'&&<p className="disclaimer">Stack Overflow contribution{e.metadata?.details?.author?' by '+e.metadata.details.author:''} · <a href="https://stackoverflow.com/help/licensing" target="_blank" rel="noopener noreferrer">CC BY-SA (version may vary)</a>. Review the question and its license at the original link.</p>}{e.source_url&&<a href={e.source_url} target="_blank" rel="noopener noreferrer">View original source ↗</a>}<button className="tiny" disabled={busy} onClick={()=>{if(window.confirm('Delete this evidence item? Saved analysis reports are historical snapshots.')) void act(()=>api('/evidence/'+e.id,{method:'DELETE'}),'Evidence deleted');}}>Delete evidence</button></article>)}</div>:<p className="empty">No evidence collected.</p>}</section></>}
   {tab==='opportunities'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">OPEN-SOURCE PROJECT CANDIDATES</span><h2>Open-source project ideas</h2><p>These themes are prompts for open-source collaboration, not community endorsements or verified demand.</p></div>{opportunities.length?<div className="list">{opportunities.map(o=><article className="list-item" key={o.id}><div className="score">{Math.round(o.score)} / 100</div><h3>{o.title}</h3><p>{o.problem}</p><small>{o.evidence_refs.length} supporting references</small></article>)}</div>:<p className="empty">Review community themes to generate ideas for useful open-source contributions.</p>}</section>}
   {tab==='jobs'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">AUTOMATION</span><h2>Background collection jobs</h2><p>Approved API access and a valid Reddit session are required.</p></div><button className="secondary" onClick={()=>void load()}>Refresh jobs</button>{jobs.length?<div className="list">{jobs.map(j=><article className="list-item job" key={j.id}><b>r/{j.subreddit}</b><span className="pill">{j.status}</span><small>Attempts: {j.attempts}</small>{j.last_error&&<p>{j.last_error}</p>}</article>)}</div>:<p className="empty">No Reddit collection jobs. Queue one from Community Discovery.</p>}</section>}
   {tab==='agents'&&<div className="two-col">
     <section className="panel"><div className="panel-heading"><span className="eyebrow">OPTIONAL ASSISTED RESEARCH</span><h2>Research jobs</h2><p>Codex or Claude can browse approved sources, supply grounded evidence and ask the Rust API to persist research progress.</p></div>
       <button className="secondary" onClick={()=>void load()}>Refresh agent state</button>
       {hostJobs.length?<div className="list">{hostJobs.map(j=><article className="list-item" key={j.id}><span className="pill">{j.status}</span><h3>{j.topic}</h3><small>Research pass: {j.pass||0}</small></article>)}</div>:<p className="empty">No host-driven jobs yet. Connect an MCP host to start one.</p>}
     </section>
     <section className="panel"><div className="panel-heading"><span className="eyebrow">SEMANTIC RESEARCH</span><h2>LLM-assisted reports</h2><p>Host-provided analysis must reference evidence IDs; the server does not invent or generate source material.</p></div>
       {hostRuns.length?<div className="list">{hostRuns.map(r=><article className="list-item" key={r.id}><span className="pill">{r.status}</span><h3>{r.name||r.topic}</h3><p>{Object.keys(r.annotations||{}).length} annotations · {(r.clusters||[]).length} clusters · {(r.opportunities||[]).length} candidates</p></article>)}</div>:<p className="empty">No host-assisted research reviews yet.</p>}
     </section>
     <section className="panel span-all"><div className="panel-heading"><span className="eyebrow">FROM RESEARCH TO CONTRIBUTION</span><h2>Opportunity workspaces</h2><p>Evidence-linked ideas, community validation, and planning for useful open-source contributions.</p></div>
       {hostWorkspaces.length?<div className="cluster-grid">{hostWorkspaces.map(w=><div className="cluster" key={w.id}><span className="pill">{w.stage}</span><h3>{w.opportunity?.title||'Untitled opportunity'}</h3><p>{w.opportunity?.problem||''}</p><small>{(w.experiments||[]).length} validation experiments</small></div>)}</div>:<p className="empty">Create opportunity workspaces through the MCP research workflow.</p>}
     </section>
   </div>}
   {tab==='integrations'&&<section className="panel"><div className="panel-heading"><span className="eyebrow">NATIVE RUST MCP / HOST INTEGRATION</span><h2>Connect Claude, Codex or another MCP host</h2><p>Create a revocable, per-account key for the local Rust MCP server. Your host controls model reasoning and optional browsing; Reddit archive collection runs in the backend; source text remains untrusted evidence.</p></div>
    <label htmlFor="mcpKeyLabel">Key description</label><input id="mcpKeyLabel" value={keyLabel} onChange={e=>setKeyLabel(e.target.value)} placeholder="My desktop MCP client"/>
    <button className="primary" disabled={!user||busy||!keyLabel.trim()} onClick={()=>void act(async()=>{const result=await api<{api_key:string}>('/mcp/keys',{method:'POST',body:JSON.stringify({label:keyLabel})});setNewKey(result.api_key);},'API key created. Copy and store it securely now.')}>Generate MCP API key</button>
    {newKey&&<div className="subpanel"><span className="eyebrow">ONE-TIME SECRET — COPY NOW</span><p>Copy this key into your MCP host configuration. It will not be shown again.</p><div className="row"><input aria-label="New API key" type="text" value={newKey} readOnly/><button className="tiny" onClick={()=>void navigator.clipboard.writeText(newKey)}>Copy</button></div></div>}
    <div className="subpanel"><span className="eyebrow">ACTIVE API KEYS</span>{keys.map(k=><div className="list-item job" key={k.id}><b>{k.label}</b><small>{new Date(k.created_at).toLocaleDateString()}</small><button className="tiny" disabled={busy} onClick={()=>void act(async()=>{await api('/mcp/keys/'+k.id,{method:'DELETE'});setNewKey('');},'API key revoked')}>Revoke</button></div>)}</div>
    <div className="subpanel"><span className="eyebrow">RUN THE MCP BRIDGE</span><p>Install Rust and run this command from the repository root with your secret set in <code>PAIN_MCP_API_KEY</code>. Keep API credentials out of committed files.</p>
     <pre className="command">cargo run --manifest-path api/Cargo.toml -- --mcp</pre>
     <p className="disclaimer">Requires a running web/API stack. The bridge exposes 58 tool schemas, including browser-free Reddit archive collection. Execution is authenticated and all workflow data stays in your PostgreSQL workspace.</p>
    </div>
   </section>}
   {tab==='settings'&&<section className="panel">
     <div className="panel-heading"><span className="eyebrow">ACCOUNT PRIVACY</span><h2>Your research data and Reddit access</h2><p>These controls operate on your account only. Research and account deletion cannot be undone.</p></div>
     <div className="subpanel">
       <h3>Export your research</h3><p>Download your projects, evidence, saved analyses and opportunity workspaces. Source-content licensing still applies to exported data. API credentials and Reddit post/comment text are excluded.</p>
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
   <footer>OPEN SOURCE DISCOVERY LAB <span>© 2026 · Local, currently non-commercial community research</span></footer>
  </main>
 </div>;
}
