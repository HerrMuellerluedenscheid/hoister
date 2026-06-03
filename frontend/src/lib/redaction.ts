// Marker the agent substitutes for sensitive env-var values and log secrets
// before anything leaves the host. Keep in sync with the agent's
// `REDACTION_MARKER` in agent/src/monitor.rs.
export const REDACTION_MARKER = '***REDACTED***';

/** True when the whole value is just the redaction marker. */
export function isRedacted(value: string): boolean {
  return value === REDACTION_MARKER;
}

/** True when the text contains the marker anywhere (e.g. inline in a log line). */
export function containsRedaction(text: string): boolean {
  return text.includes(REDACTION_MARKER);
}
