import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import Home from '../app/page';
import type { Opportunity, OpportunityBrief } from '../lib/types';
import { mockWorkspace } from './fixtures';

const opportunity: Opportunity = {
  id: 'candidate-one',
  run_id: 'run-one',
  project_id: null,
  title: 'Manual workflow',
  problem: 'Two independent fictional discussions.',
  score: 50,
  evidence_refs: ['evidence-one'],
  created_at: '2026-10-01T00:00:00Z',
  decision: 'needs_validation',
  existing_solutions: 'Existing manual spreadsheet',
  validation_notes: 'Verify with at least three independent contributors.',
};

async function openReview(brief?: OpportunityBrief) {
  const fetchMock = mockWorkspace({ opportunities: [opportunity], opportunityBrief: brief || null });
  render(<Home />);
  const actor = userEvent.setup();
  await screen.findByRole('button', { name: 'Sign out' });
  await actor.click(within(screen.getByRole('navigation')).getByRole('button', { name: /Project Ideas/ }));
  await actor.click(await screen.findByRole('button', { name: 'Review evidence and decision' }));
  return { actor, fetchMock };
}

describe('manual opportunity review', () => {
  it('loads source-linked evidence and saves an owner-scoped decision', async () => {
    const { actor, fetchMock } = await openReview();
    expect(await screen.findByText('Fictional manual workflow evidence')).toBeVisible();
    expect(screen.getByRole('link', { name: 'Read original source' })).toHaveAttribute(
      'href', 'https://example.com/fictional-one',
    );
    expect(screen.getByText(/Fewer than three independent/)).toBeVisible();
    await actor.selectOptions(screen.getByLabelText('Your next decision'), 'investigating');
    await actor.click(screen.getByRole('button', { name: 'Save manual review' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith(
      '/api/opportunities/candidate-one/review',
      expect.objectContaining({
        method: 'PUT', credentials: 'same-origin',
        body: JSON.stringify({
          decision: 'investigating',
          existing_solutions: 'Existing manual spreadsheet',
          validation_notes: 'Verify with at least three independent contributors.',
        }),
      }),
    ));
  });

  it('does not render unsafe or expired evidence as a clickable source', async () => {
    await openReview({
      title: 'Manual workflow',
      referenced_count: 2,
      caveat: 'Historical research',
      cluster: {
        supporting_evidence: [
          { evidence_id: 'a', available: true, excerpt: 'Unsafe link test',
            source_url: 'javascript:alert(1)' },
          { evidence_id: 'b', available: false },
        ],
      },
    });
    expect(await screen.findByText('Unsafe link test')).toBeVisible();
    expect(screen.queryByRole('link', { name: 'Read original source' })).not.toBeInTheDocument();
    expect(screen.getByText(/Supporting evidence is missing/)).toBeVisible();
  });

  it('requires a justification before allowing a ready-to-build decision', async () => {
    const { actor } = await openReview();
    await actor.selectOptions(screen.getByLabelText('Your next decision'), 'ready_to_build');
    await actor.clear(screen.getByLabelText('Validation notes'));
    expect(screen.getByRole('button', { name: 'Save manual review' })).toBeDisabled();
    await actor.type(screen.getByLabelText('Validation notes'), 'We independently validated this issue.');
    expect(screen.getByRole('button', { name: 'Save manual review' })).toBeEnabled();
  });
});
