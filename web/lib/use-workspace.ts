'use client';
import { useCallback, useEffect, useState } from 'react';
import { api, loadList } from './api';
import type {
  User,
  Project,
  Evidence,
  Run,
  Job,
  HostRun,
  HostWorkspace,
  HostJob,
  ApiKey,
  Opportunity,
  SourceOption,
  SourceCatalog,
} from './types';

// Keep shared data and form drafts mounted when a view changes.
export function useWorkspace() {
  const [user, setUser] = useState<User | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [items, setItems] = useState<Evidence[]>([]);
  const [runs, setRuns] = useState<Run[]>([]);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [opportunities, setOpportunities] = useState<Opportunity[]>([]);
  const [selected, setSelected] = useState('');
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [hostRuns, setHostRuns] = useState<HostRun[]>([]);
  const [hostWorkspaces, setHostWorkspaces] = useState<HostWorkspace[]>([]);
  const [hostJobs, setHostJobs] = useState<HostJob[]>([]);
  const [keyLabel, setKeyLabel] = useState('My MCP client');
  const [newKey, setNewKey] = useState('');
  const [subreddit, setSubreddit] = useState('opensource');
  const [localToken, setLocalToken] = useState('');
  const [catalog, setCatalog] = useState<SourceOption[]>([]);
  const [source, setSource] = useState('github');
  const [sourceQuery, setSourceQuery] = useState('feature request');
  const [sourceTarget, setSourceTarget] = useState('');
  const [sourceResult, setSourceResult] = useState('');
  const [archiveRanking, setArchiveRanking] = useState('top');
  const [archiveDays, setArchiveDays] = useState(30);
  const [archivePost, setArchivePost] = useState('');

  const [threadId, setThreadId] = useState('');
  const [manualTitle, setManualTitle] = useState('');
  const [manualBody, setManualBody] = useState('');
  const [manualUrl, setManualUrl] = useState('');
  const [topic, setTopic] = useState('Recurring community needs that open-source projects could address');
  const [projectName, setProjectName] = useState('');
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState('');
  const load = useCallback(async () => {
    try {
      const me = await api<User>('/me');
      setUser(me);
    } catch (e) {
      setUser(null);
      setProjects([]);
      setItems([]);
      setRuns([]);
      setJobs([]);
      setOpportunities([]);
      setKeys([]);
      setHostRuns([]);
      setHostWorkspaces([]);
      setHostJobs([]);
      setNotice(e instanceof Error ? e.message : 'Unable to authenticate');
      return;
    }
    // Keep the authenticated session visible when a separate research endpoint is unavailable.
    const results = await Promise.all([
      loadList(api<Project[]>('/projects'), setProjects),
      loadList(api<Evidence[]>('/evidence?limit=200'), setItems),
      loadList(api<Run[]>('/research/runs'), setRuns),
      loadList(api<Job[]>('/research/jobs'), setJobs),
      loadList(
        api<Opportunity[]>('/opportunities' + (selected ? '?project_id=' + encodeURIComponent(selected) : '')),
        setOpportunities,
      ),
      loadList(api<ApiKey[]>('/mcp/keys'), setKeys),
      loadList(api<HostRun[]>('/mcp/runs'), setHostRuns),
      loadList(api<HostWorkspace[]>('/mcp/workspaces'), setHostWorkspaces),
      loadList(api<HostJob[]>('/mcp/research-jobs'), setHostJobs),
    ]);
    if (results.some((success) => !success))
      setNotice('Some research sections could not load. Your session is still active; refresh to retry.');
  }, [selected]);
  useEffect(() => {
    void load();
  }, [load]);
  useEffect(() => {
    void api<SourceCatalog>('/sources')
      .then((v) => setCatalog(v.sources))
      .catch(() => setCatalog([]));
  }, []);
  const chosenSource = catalog.find((x) => x.id === source);

  async function act(action: () => Promise<unknown>, message: string) {
    setBusy(true);
    setNotice('');
    try {
      await action();
      setNotice(message);
      await load();
    } catch (e) {
      setNotice(e instanceof Error ? e.message : 'Operation failed');
    } finally {
      setBusy(false);
    }
  }
  const project_id = selected || null;
  const latest = runs.find((r) => !selected || r.project_id === selected);

  const context = { user, busy, act };
  const scope = { projects, selected, setSelected, project_id };
  const projectForm = { projectName, setProjectName };
  return {
    user,
    notice,
    context,
    scope,
    projectForm,
    login: { localToken, setLocalToken },
    sources: {
      catalog,
      chosenSource,
      source,
      setSource,
      sourceQuery,
      setSourceQuery,
      sourceTarget,
      setSourceTarget,
      sourceResult,
      setSourceResult,
      archiveRanking,
      setArchiveRanking,
      archiveDays,
      setArchiveDays,
      archivePost,
      setArchivePost,
    },
    evidence: { items, manualTitle, setManualTitle, manualBody, setManualBody, manualUrl, setManualUrl },
    evidenceScope: { project_id },
    analysis: {
      subreddit,
      setSubreddit,
      threadId,
      setThreadId,
      topic,
      setTopic,
      items,
      latest,
      projectEditor: { ...context, ...scope, ...projectForm },
    },
    ideas: { opportunities },
    jobView: { jobs, load },
    assisted: { hostJobs, hostRuns, hostWorkspaces, load },
    integrations: { keys, keyLabel, setKeyLabel, newKey, setNewKey },
    counts: { items: items.length, runs: runs.length, opportunities: opportunities.length, jobs: jobs.length },
    signOut: () =>
      act(
        () =>
          api('/auth/logout', { method: 'POST' }).then(() => {
            setUser(null);
            setProjects([]);
            setItems([]);
            setRuns([]);
            setJobs([]);
            setOpportunities([]);
          }),
        'Disconnected',
      ),
  };
}

export type Workspace = ReturnType<typeof useWorkspace>;
