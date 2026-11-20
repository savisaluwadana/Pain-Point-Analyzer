'use client';
import { useEffect, useState } from 'react';
import type { Tab } from '../lib/types';
import { useWorkspace } from '../lib/use-workspace';
import WorkspaceLayout from '../components/WorkspaceLayout';
import Analyze from '../components/Analyze';
import Sources from '../components/Sources';
import EvidenceLibrary from '../components/EvidenceLibrary';
import ProjectIdeas from '../components/ProjectIdeas';
import Jobs from '../components/Jobs';
import AssistedResearch from '../components/AssistedResearch';
import McpIntegrations from '../components/McpIntegrations';
import Settings from '../components/Settings';
import LoginForm from '../components/LoginForm';

export default function Home() {
  const workspace = useWorkspace();
  const [tab, setTab] = useState<Tab>('sources');
  useEffect(() => {
    const readTab = () => {
      const value = new URLSearchParams(window.location.search).get('tab');
      const tabs: Tab[] = [
        'sources',
        'discover',
        'evidence',
        'opportunities',
        'jobs',
        'agents',
        'integrations',
        'settings',
      ];
      setTab(tabs.find((item) => item === value) || 'sources');
    };
    readTab();
    window.addEventListener('popstate', readTab);
    return () => window.removeEventListener('popstate', readTab);
  }, []);
  function navigate(next: Tab) {
    setTab(next);
    const url = new URL(window.location.href);
    url.searchParams.set('tab', next);
    window.history.pushState(null, '', url);
  }
  return (
    <WorkspaceLayout
      tab={tab}
      setTab={navigate}
      user={workspace.user}
      notice={workspace.notice}
      loadError={workspace.loadError}
      loading={workspace.loading}
      busy={workspace.context.busy}
      onRefresh={workspace.load}
      counts={workspace.counts}
      onSignOut={workspace.signOut}
      login={!workspace.user && !workspace.loading && <LoginForm {...workspace.context} {...workspace.login} />}
    >
      {tab === 'sources' && <Sources {...workspace.context} {...workspace.scope} {...workspace.sources} />}
      {tab === 'discover' && <Analyze {...workspace.context} {...workspace.scope} {...workspace.analysis} />}
      {tab === 'evidence' && <EvidenceLibrary {...workspace.context} {...workspace.scope} {...workspace.evidence} />}
      {tab === 'opportunities' && <ProjectIdeas {...workspace.ideas} {...workspace.scope} />}
      {tab === 'jobs' && <Jobs {...workspace.jobView} />}
      {tab === 'agents' && <AssistedResearch {...workspace.assisted} />}
      {tab === 'integrations' && <McpIntegrations {...workspace.context} {...workspace.integrations} />}
      {tab === 'settings' && <Settings {...workspace.context} />}
    </WorkspaceLayout>
  );
}
