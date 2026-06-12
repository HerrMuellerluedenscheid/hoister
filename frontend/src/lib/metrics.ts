import type { MetricPointResponse } from '../bindings/MetricPointResponse';

export type ChartPoint = { t: number; v: number };

/** Map a gauge field (cpu%, memory bytes) straight to chart points. */
export function gaugeSeries(
  points: MetricPointResponse[],
  field: (p: MetricPointResponse) => number
): ChartPoint[] {
  return points.map((p) => ({ t: new Date(p.recorded_at).getTime(), v: field(p) }));
}

/**
 * Network and storage are cumulative byte counters since container start, so a
 * raw plot would just ramp upward. Differentiate against the previous sample to
 * recover throughput in bytes/second. The first sample has no predecessor and
 * a counter that goes backwards (container restart) is clamped to 0.
 */
export function rateSeries(
  points: MetricPointResponse[],
  field: (p: MetricPointResponse) => number
): ChartPoint[] {
  const out: ChartPoint[] = [];
  for (let i = 1; i < points.length; i++) {
    const t = new Date(points[i].recorded_at).getTime();
    const prevT = new Date(points[i - 1].recorded_at).getTime();
    const seconds = (t - prevT) / 1000;
    if (seconds <= 0) continue;
    const delta = field(points[i]) - field(points[i - 1]);
    out.push({ t, v: delta > 0 ? delta / seconds : 0 });
  }
  return out;
}
