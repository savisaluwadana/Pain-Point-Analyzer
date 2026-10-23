import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Home from '../app/page';
import { evidence, mockWorkspace, project, run } from './fixtures';

async function openWorkspace(options?: Parameters<typeof mockWorkspace>[0]) {
  const fetchMock = mockWorkspace(options);
  render(<Home />);
  await screen.findByRole('button', { name: 'Sign out' });
  // Wait for data loading as well as authentication.
  await screen.findByRole('option', { name: project.name });
  return { fetchMock, actor: userEvent.setup() };
}

async function navigate(actor: ReturnType<typeof userEvent.setup>, name: RegExp) {
  await actor.click(within(screen.getByRole('navigation')).getByRole('button', { name }));
}

describe('local login', () => {
  it('keeps the token private, surfaces errors, and opens the workspace after successful login', async () => {
    const fetchMock = mockWorkspace({ signedIn: false });
    const actor = userEvent.setup();
    render(<Home />);
    const token = screen.getByLabelText('Local access token');
    const login = screen.getByRole('button', { name: /Sign in locally/ });
    expect(token).toHaveAttribute('type', 'password');
    expect(token).toHaveAttribute('autocomplete', 'off');
    expect(login).toBeDisabled();
    await screen.findByText('Sign in to the research workspace');
    await actor.type(token, 'wrong-token');
    await actor.click(login);
    expect(await screen.findByText('Invalid local access token')).toBeVisible();
    expect(token).toHaveValue('wrong-token');
    await actor.clear(token);
    await actor.type(token, 'a'.repeat(64));
    await actor.click(login);
    expect(await screen.findByRole('button', { name: 'Sign out' })).toBeVisible();
    expect(screen.queryByLabelText('Local access token')).not.toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/auth/local',
      expect.objectContaining({
        method: 'POST',
        credentials: 'same-origin',
        body: JSON.stringify({ token: 'a'.repeat(64) }),
      }),
    );
    expect(window.localStorage.length).toBe(0);
  });

  it('retains authentication when an optional research endpoint fails, then clears data on logout', async () => {
    const { actor } = await openWorkspace({ failedPath: '/api/mcp/runs' });
    expect(await screen.findByText(/Some research sections could not load/)).toBeVisible();
    await navigate(actor, /Evidence Library/);
    expect(screen.getByRole('heading', { name: evidence[0].title })).toBeVisible();
    await actor.click(screen.getByRole('button', { name: 'Sign out' }));
    expect(await screen.findByLabelText('Local access token')).toBeVisible();
    await waitFor(() => expect(screen.queryByRole('article')).not.toBeInTheDocument());
  });
});

describe('evidence library', () => {
  it('renders excerpts, title fallbacks, original links and source attribution', async () => {
    const { actor } = await openWorkspace();
    await navigate(actor, /Evidence Library/);
    const articles = screen.getAllByRole('article');
    expect(articles).toHaveLength(2);
    expect(within(articles[0]).getByRole('heading', { name: evidence[0].title })).toBeVisible();
    expect(within(articles[1]).getByRole('heading', { name: evidence[1].source_id })).toBeVisible();
    expect(within(articles[1]).getByText(evidence[1].body.slice(0, 260).trim())).toBeVisible();
    expect(within(articles[1]).getByText(/Fictional contributor/)).toBeVisible();
    expect(within(articles[1]).getByRole('link', { name: /CC BY-SA/ })).toHaveAttribute(
      'href',
      'https://stackoverflow.com/help/licensing',
    );
    expect(within(articles[0]).getByRole('link', { name: /View original source/ })).toHaveAttribute(
      'href',
      evidence[0].source_url,
    );
    expect(within(articles[0]).getByRole('link')).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('keeps the evidence deletion confirmation and sends the selected evidence ID', async () => {
    const { actor, fetchMock } = await openWorkspace();
    await navigate(actor, /Evidence Library/);
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);
    await actor.click(screen.getAllByRole('button', { name: 'Delete evidence' })[0]);
    expect(fetchMock.mock.calls.some(([, options]) => options?.method === 'DELETE')).toBe(false);
    confirm.mockReturnValue(true);
    await actor.click(screen.getAllByRole('button', { name: 'Delete evidence' })[0]);
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        '/api/evidence/evidence-one',
        expect.objectContaining({ method: 'DELETE', credentials: 'same-origin' }),
      ),
    );
  });

  it('renders the empty state when there is no evidence', async () => {
    const { actor } = await openWorkspace({ empty: true });
    await navigate(actor, /Evidence Library/);
    expect(screen.getByText('No evidence collected.')).toBeVisible();
    expect(screen.queryByRole('article')).not.toBeInTheDocument();
  });
});

describe('analysis and navigation', () => {
  it('renders the existing scores and caveat, and submits the question with its project scope', async () => {
    const { actor, fetchMock } = await openWorkspace();
    await navigate(actor, /Analyze & Discover/);
    expect(screen.getByRole('heading', { name: run.topic })).toBeVisible();
    expect(screen.getByText('42/100')).toBeVisible();
    expect(screen.getByText('2 independent stories · 2 signals')).toBeVisible();
    expect(screen.getByText(run.report.caveat)).toBeVisible();
    await actor.selectOptions(screen.getByLabelText('Research project'), project.id);
    const question = screen.getByLabelText('Research question');
    await actor.clear(question);
    await actor.type(question, 'How can fictional reporting improve?');
    await actor.click(screen.getByRole('button', { name: /Run evidence analysis/ }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        '/api/research/analyze',
        expect.objectContaining({
          method: 'POST',
          credentials: 'same-origin',
          body: JSON.stringify({ topic: 'How can fictional reporting improve?', project_id: project.id }),
        }),
      ),
    );
  });

  it('disables analysis until evidence exists', async () => {
    const { actor } = await openWorkspace({ empty: true });
    await navigate(actor, /Analyze & Discover/);
    expect(screen.getByRole('button', { name: /Run evidence analysis/ })).toBeDisabled();
    expect(screen.getByRole('heading', { name: 'No research report yet' })).toBeVisible();
  });

  it('preserves drafts and the selected project across tab switches', async () => {
    const { actor } = await openWorkspace();
    const query = screen.getByLabelText('Search topic or phrase');
    await actor.clear(query);
    await actor.type(query, 'Fictional source draft');
    await actor.selectOptions(screen.getByLabelText('Research project'), project.id);
    await navigate(actor, /Evidence Library/);
    await actor.type(screen.getByLabelText('Evidence title'), 'Fictional evidence draft');
    await navigate(actor, /Analyze & Discover/);
    await actor.type(screen.getByLabelText('Project name'), 'Fictional project draft');
    expect(screen.getByLabelText('Research project')).toHaveValue(project.id);
    await navigate(actor, /Community Sources/);
    expect(screen.getByLabelText('Search topic or phrase')).toHaveValue('Fictional source draft');
    await navigate(actor, /Evidence Library/);
    expect(screen.getByLabelText('Evidence title')).toHaveValue('Fictional evidence draft');
    await navigate(actor, /Analyze & Discover/);
    expect(screen.getByLabelText('Project name')).toHaveValue('Fictional project draft');
  });

  it('keeps bounded archive request settings and preserves the one-time MCP key across tabs', async () => {
    const { actor, fetchMock } = await openWorkspace();
    await actor.selectOptions(screen.getByLabelText('Source'), 'arctic-shift');
    await actor.type(screen.getByLabelText('Subreddit (without r/)'), 'fictionalcommunity');
    await actor.click(screen.getByRole('button', { name: /Collect source evidence/ }));
    await waitFor(() =>
      expect(fetchMock).toHaveBeenCalledWith(
        '/api/sources/collect',
        expect.objectContaining({
          body: JSON.stringify({
            source: 'arctic-shift',
            query: '',
            target: 'fictionalcommunity',
            project_id: null,
            limit: 10,
            days: 30,
            ranking: 'top',
            sample_size: 100,
          }),
        }),
      ),
    );
    await navigate(actor, /MCP Integrations/);
    await actor.click(screen.getByRole('button', { name: 'Generate MCP API key' }));
    expect(await screen.findByLabelText('New API key')).toHaveValue('pil_fictional_test_key');
    await navigate(actor, /Privacy & Settings/);
    await navigate(actor, /MCP Integrations/);
    expect(screen.getByLabelText('New API key')).toHaveValue('pil_fictional_test_key');
  });
});
