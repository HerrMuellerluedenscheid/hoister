import { describe, it, expect } from 'vitest';
import { isStale, STALE_AFTER_MS } from './staleness';

describe('isStale', () => {
  const now = Date.parse('2026-01-01T00:00:00Z');
  const ago = (ms: number) => new Date(now - ms).toISOString();

  it('leaves room for a missed report before flagging', () => {
    expect(isStale(ago(60_000), now)).toBe(false);
  });

  it('flags updates older than the threshold', () => {
    expect(isStale(ago(STALE_AFTER_MS + 1), now)).toBe(true);
  });
});
