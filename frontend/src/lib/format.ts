// Shared formatters for the resource-usage metrics surfaced on the containers
// dashboard and detail page. Kept dependency-free and binary-unit based so the
// figures line up with what `docker stats` reports.

const BYTE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

/** Human-readable byte size, e.g. `1.4 GB`. Uses 1024-based units. */
export function formatBytes(bytes: number, fractionDigits = 1): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B';
  }
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), BYTE_UNITS.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  // Whole bytes never need decimals.
  const digits = exponent === 0 ? 0 : fractionDigits;
  return `${value.toFixed(digits)} ${BYTE_UNITS[exponent]}`;
}

/** Throughput in bytes-per-second, e.g. `2.3 MB/s`. */
export function formatRate(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

/** CPU percentage. Values can exceed 100 on multi-core hosts, matching `docker stats`. */
export function formatPercent(pct: number, fractionDigits = 1): string {
  if (!Number.isFinite(pct) || pct < 0) {
    return '0%';
  }
  return `${pct.toFixed(fractionDigits)}%`;
}

/** Memory usage as a fraction of the limit, `null` when the limit is unset (0 = unlimited). */
export function memoryFraction(used: number, limit: number): number | null {
  if (!Number.isFinite(limit) || limit <= 0) {
    return null;
  }
  return Math.min(used / limit, 1);
}
