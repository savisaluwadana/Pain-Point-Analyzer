'use client';
import { useState } from 'react';
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
  return (
    <WorkspaceLayout
      tab={tab}
      setTab={setTab}
      user={workspace.user}
      notice={workspace.notice}
      counts={workspace.counts}
      onSignOut={workspace.signOut}
      login={!workspace.user && <LoginForm {...workspace.context} {...workspace.login} />}
    >
      {tab === 'sources' && <Sources {...workspace.context} {...workspace.scope} {...workspace.sources} />}
      {tab === 'discover' && <Analyze {...workspace.context} {...workspace.scope} {...workspace.analysis} />}
      {tab === 'evidence' && (
        <EvidenceLibrary {...workspace.context} {...workspace.evidenceScope} {...workspace.evidence} />
      )}
      {tab === 'opportunities' && <ProjectIdeas {...workspace.ideas} />}
      {tab === 'jobs' && <Jobs {...workspace.jobView} />}
      {tab === 'agents' && <AssistedResearch {...workspace.assisted} />}
      {tab === 'integrations' && <McpIntegrations {...workspace.context} {...workspace.integrations} />}
      {tab === 'settings' && <Settings {...workspace.context} />}
    </WorkspaceLayout>
  );
}
