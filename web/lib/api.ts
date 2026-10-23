export async function api<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch('/api' + path, {
    ...options,
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json', ...options?.headers },
  });
  const payload: unknown = await response.json().catch(() => ({}));
  if (!response.ok) {
    const message =
      typeof payload === 'object' &&
      payload !== null &&
      'error' in payload &&
      typeof payload.error === 'string' &&
      payload.error
        ? payload.error
        : 'Request failed';
    throw new Error(message);
  }
  return payload as T;
}

// A single optional endpoint failure must not invalidate an otherwise valid login.
export async function loadList<T>(request: Promise<T[]>, apply: (items: T[]) => void): Promise<boolean> {
  try {
    apply(await request);
    return true;
  } catch {
    apply([]);
    return false;
  }
}
