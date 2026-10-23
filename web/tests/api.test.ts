import { describe, expect, it, vi } from 'vitest';
import { api, loadList } from '../lib/api';

describe('API error handling', () => {
  it.each([
    [Response.json({ error: 'Provider refused access' }, { status: 403 }), 'Provider refused access'],
    [new Response('not JSON', { status: 502 }), 'Request failed'],
    [Response.json(null, { status: 500 }), 'Request failed'],
    [Response.json({ error: { unexpected: true } }, { status: 500 }), 'Request failed'],
  ])('returns a readable error for failed responses', async (response, message) => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(response));
    await expect(api('/evidence')).rejects.toThrow(message);
  });

  it('propagates network failures and isolates optional list failures', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Connection unavailable')));
    await expect(api('/me')).rejects.toThrow('Connection unavailable');
    const apply = vi.fn();
    expect(await loadList(api<unknown[]>('/evidence'), apply)).toBe(false);
    expect(apply).toHaveBeenCalledWith([]);
  });
});
