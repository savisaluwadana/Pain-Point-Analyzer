'use client';
import { useCallback, useEffect, useRef, useState } from 'react';
import { api, ApiError, loadList } from './api';
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
  const [selected, setSelectedState] = useState('');
  const selectedRef = useRef('');
  const generation = useRef(0);
  const actionPending = useRef(false);
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
  const [loadError, setLoadError] = useState('');
  const [loading, setLoading] = useState(true);
  const [analysisLimit, setAnalysisLimit] = useState(1000);
  const setSelected = useCallback((value: string) => {
    if (value === selectedRef.current) return;
    selectedRef.current = value;
    generation.current += 1;
    setSelectedState(value);
    setItems([]);
    setOpportunities([]);
    setLoading(true);
  }, []);

  const clearSession = useCallback(() => {
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
    setNewKey('');
    setLocalToken('');
    setManualTitle('');
    setManualBody('');
    setManualUrl('');
    setProjectName('');
    setSourceResult('');
    setSourceQuery('feature request');
    setSourceTarget('');
    setSource('github');
    setArchivePost('');
    setArchiveRanking('top');
    setArchiveDays(30);
    setAnalysisLimit(1000);
    setSubreddit('opensource');
    setThreadId('');
    setTopic('Recurring community needs that open-source projects could address');
    setKeyLabel('My MCP client');
    selectedRef.current = '';
    setSelectedState('');
  }, []);
  const load = useCallback(async () => {
    const current = ++generation.current;
    const scope = selectedRef.current;
    const active = () => generation.current === current;
    const apply =
      <T>(setter: (value: T) => void) =>
      (value: T) => {
        if (active()) setter(value);
      };
    setLoading(true);
    setLoadError('');
    const catalogRequest = api<SourceCatalog>('/sources')
      .then((v) => {
        if (!Array.isArray(v?.sources)) throw new Error('Invalid source catalog');
        apply(setCatalog)(v.sources);
        return true;
      })
      .catch(() => {
        apply(setCatalog)([]);
        return false;
      });
    try {
      const me = await api<User>('/me');
      if (!me?.id || typeof me.reddit_username !== 'string') throw new Error('Invalid session response');
      if (!active()) return;
      setUser(me);
    } catch (e) {
      if (!active()) return;
      if (e instanceof ApiError && e.status === 401) clearSession();
      setLoadError(e instanceof Error ? e.message : 'Unable to authenticate');
      setLoading(false);
      return;
    }
    // Keep the authenticated session visible when a separate research endpoint is unavailable.
    const results = await Promise.all([
      loadList(api<Project[]>('/projects'), apply(setProjects)),
      loadList(
        api<Evidence[]>('/evidence?limit=200' + (scope ? '&project_id=' + encodeURIComponent(scope) : '')),
        apply(setItems),
      ),
      loadList(api<Run[]>('/research/runs'), apply(setRuns)),
      loadList(api<Job[]>('/research/jobs'), apply(setJobs)),
      loadList(
        api<Opportunity[]>('/opportunities' + (scope ? '?project_id=' + encodeURIComponent(scope) : '')),
        apply(setOpportunities),
      ),
      loadList(api<ApiKey[]>('/mcp/keys'), apply(setKeys)),
      loadList(api<HostRun[]>('/mcp/runs'), apply(setHostRuns)),
      loadList(api<HostWorkspace[]>('/mcp/workspaces'), apply(setHostWorkspaces)),
      loadList(api<HostJob[]>('/mcp/research-jobs'), apply(setHostJobs)),
      catalogRequest,
    ]);
    if (!active()) return;
    if (results.some((success) => !success))
      setLoadError('Some research sections could not load. Your session is still active; refresh to retry.');
    setLoading(false);
  }, [clearSession]);
  useEffect(() => {
    void load();
    return () => {
      generation.current += 1;
    };
  }, [load, selected]);
  const chosenSource = catalog.find((x) => x.id === source);

  async function act(action: () => Promise<unknown>, message: string) {
    if (actionPending.current) return;
    actionPending.current = true;
    setBusy(true);
    setNotice('');
    try {
      await action();
      setNotice(message);
      await load();
    } catch (e) {
      setNotice(e instanceof Error ? e.message : 'Operation failed');
    } finally {
      actionPending.current = false;
      setBusy(false);
    }
  }
  const project_id = selected || null;
  const latest = runs.find((r) => !selected || r.project_id === selected);

  const context = { user, busy: busy || loading, act };
  const scope = { projects, selected, setSelected, project_id };
  const projectForm = { projectName, setProjectName };
  return {
    user,
    notice,
    loadError,
    loading,
    load,
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
    analysis: {
      subreddit,
      setSubreddit,
      threadId,
      setThreadId,
      topic,
      setTopic,
      analysisLimit,
      setAnalysisLimit,
      items,
      latest,
      projectEditor: { ...context, ...scope, ...projectForm },
    },
    ideas: { opportunities, busy: busy || loading, act },
    jobView: { jobs, load },
    assisted: { hostJobs, hostRuns, hostWorkspaces, load },
    integrations: { keys, keyLabel, setKeyLabel, newKey, setNewKey },
    counts: { items: items.length, runs: runs.length, opportunities: opportunities.length, jobs: jobs.length },
    signOut: () =>
      act(
        () =>
          api('/auth/logout', { method: 'POST' }).then(() => {
            generation.current += 1;
            clearSession();
          }),
        'Disconnected',
      ),
  };
}

export type Workspace = ReturnType<typeof useWorkspace>;
