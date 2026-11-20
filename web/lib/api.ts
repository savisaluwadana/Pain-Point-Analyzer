export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

export function safeSourceUrl(value?: string) {
  try {
    const url = new URL(value || '');
    return ['http:', 'https:'].includes(url.protocol) ? url.href : undefined;
  } catch {
    return undefined;
  }
}

export async function api<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch('/api' + path, {
    ...options,
    credentials: 'same-origin',
    headers: { 'Content-Type': 'application/json', ...options?.headers },
  });
  let payload: unknown;
  try {
    payload = await response.json();
  } catch {
    if (response.ok && response.status !== 204)
      throw new ApiError('Server returned an invalid response', response.status);
  }
  if (!response.ok) {
    const message =
      typeof payload === 'object' &&
      payload !== null &&
      'error' in payload &&
      typeof payload.error === 'string' &&
      payload.error
        ? payload.error
        : 'Request failed';
    throw new ApiError(message, response.status);
  }
  return payload as T;
}

// A single optional endpoint failure must not invalidate an otherwise valid login.
export async function loadList<T>(request: Promise<T[]>, apply: (items: T[]) => void): Promise<boolean> {
  try {
    const items = await request;
    if (!Array.isArray(items)) throw new Error('Expected a list');
    apply(items);
    return true;
  } catch {
    apply([]);
    return false;
  }
}
