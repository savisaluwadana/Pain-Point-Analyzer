import { vi } from 'vitest';
import type { Evidence, Project, Run, User } from '../lib/types';

export const user: User = {
  id: 'local-user',
  reddit_username: 'local-researcher',
  reddit_connected: false,
  local_account: true,
};
export const project: Project = {
  id: 'project-one',
  name: 'Fictional volunteer workflows',
  description: '',
  created_at: '2026-10-01T00:00:00Z',
};
export const evidence: Evidence[] = [
  {
    id: 'evidence-one',
    source_kind: 'manual',
    source_id: 'fictional-one',
    source_url: 'https://example.com/fictional-one',
    title: 'Fictional reporting workload',
    body: 'A fictional volunteer copies a report into a spreadsheet every week.',
    collected_at: '2026-10-01T00:00:00Z',
  },
  {
    id: 'evidence-two',
    source_kind: 'stackoverflow',
    source_id: 'fictional-two',
    source_url: 'https://example.com/fictional-two',
    title: '',
    body: 'Fictional test excerpt. '.repeat(20),
    metadata: { details: { author: 'Fictional contributor' } },
    collected_at: '2026-10-01T00:00:00Z',
  },
];
export const run: Run = {
  id: 'run-one',
  project_id: project.id,
  topic: 'Fictional reporting research',
  created_at: '2026-10-01T00:00:00Z',
  report: {
    evidence_scanned: 2,
    independent_stories: 2,
    clusters: [
      {
        category: 'Manual workflow',
        independent_stories: 2,
        evidence_count: 2,
        severity_score: 42,
        examples: [],
      },
    ],
    caveat: 'Fictional heuristic results require independent review.',
  },
};

// Mock only the transport; tests exercise the real fetch helper, hooks and view handlers.
export function mockWorkspace({ signedIn = true, empty = false, failedPath = '', reportRun = run } = {}) {
  let authenticated = signedIn;
  const fetchMock = vi.fn(async (input: RequestInfo | URL, options?: RequestInit) => {
    const path = new URL(String(input), 'http://localhost:3000').pathname;
    if (path === '/api/auth/local') {
      const token = JSON.parse(String(options?.body)).token;
      if (token !== 'a'.repeat(64)) return Response.json({ error: 'Invalid local access token' }, { status: 401 });
      authenticated = true;
      return Response.json({ ok: true });
    }
    if (path === '/api/auth/logout') {
      authenticated = false;
      return Response.json({ ok: true });
    }
    if (path === '/api/sources') {
      return Response.json({
        sources: [
          { id: 'github', name: 'GitHub Issues', mode: 'keyword', ready: true, note: 'Fictional connector fixture.' },
          {
            id: 'arctic-shift',
            name: 'Reddit archive (Arctic Shift)',
            mode: 'subreddit',
            ready: true,
            note: 'Fictional archive fixture.',
          },
        ],
      });
    }
    if (!authenticated) return Response.json({ error: 'Sign in to the research workspace' }, { status: 401 });
    if (path === failedPath) return Response.json({ error: 'Fixture unavailable' }, { status: 503 });
    if (options?.method === 'POST' || options?.method === 'DELETE') {
      if (path === '/api/research/analyze') return Response.json({ run_id: run.id, report: run.report });
      if (path === '/api/mcp/keys') return Response.json({ api_key: 'pil_fictional_test_key' });
      if (path === '/api/sources/collect')
        return Response.json({ saved: 2, fetched: 10, eligible: 5, note: 'Fixture.' });
      return Response.json({ ok: true });
    }
    const data: Record<string, unknown> = {
      '/api/me': user,
      '/api/projects': [project],
      '/api/evidence': empty ? [] : evidence,
      '/api/research/runs': empty ? [] : [reportRun],
      '/api/research/jobs': [],
      '/api/opportunities': [],
      '/api/mcp/keys': [],
      '/api/mcp/runs': [],
      '/api/mcp/workspaces': [],
      '/api/mcp/research-jobs': [],
    };
    if (!(path in data)) throw new Error(`Unexpected test endpoint: ${path}`);
    return Response.json(data[path]);
  });
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}
