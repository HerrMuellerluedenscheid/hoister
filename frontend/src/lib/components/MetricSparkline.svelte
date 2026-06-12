<script lang="ts">
  // Dependency-free inline-SVG sparkline. Renders one or more series on a
  // shared y-scale (e.g. network rx + tx together) over a fixed viewBox; the
  // browser scales the vector to the container width.

  export type Series = {
    label: string;
    /** CSS color used for the stroke and (faintly) the area fill. */
    color: string;
    points: { t: number; v: number }[];
  };

  let {
    title,
    series,
    format,
    unit = ''
  }: {
    title: string;
    series: Series[];
    /** Formats a value for the headline/peak readouts. */
    format: (v: number) => string;
    /** Optional suffix shown after the title, e.g. a memory limit. */
    unit?: string;
  } = $props();

  // Fixed coordinate space; preserveAspectRatio="none" lets it stretch to fit.
  const W = 100;
  const H = 32;

  const allPoints = $derived(series.flatMap((s) => s.points));
  const hasData = $derived(allPoints.length > 0);

  const tMin = $derived(hasData ? Math.min(...allPoints.map((p) => p.t)) : 0);
  const tMax = $derived(hasData ? Math.max(...allPoints.map((p) => p.t)) : 1);
  // Headroom above the peak so the line never clips the top edge.
  const vMax = $derived(Math.max(...allPoints.map((p) => p.v), 0) * 1.1 || 1);

  function x(t: number): number {
    return tMax === tMin ? W : ((t - tMin) / (tMax - tMin)) * W;
  }
  function y(v: number): number {
    return H - (v / vMax) * H;
  }

  function linePath(points: { t: number; v: number }[]): string {
    if (points.length === 0) return '';
    return points.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.t)},${y(p.v)}`).join(' ');
  }

  function areaPath(points: { t: number; v: number }[]): string {
    if (points.length === 0) return '';
    const top = points.map((p) => `L${x(p.t)},${y(p.v)}`).join(' ');
    return `M${x(points[0].t)},${H} ${top} L${x(points[points.length - 1].t)},${H} Z`;
  }

  function latest(s: Series): number {
    return s.points.length ? s.points[s.points.length - 1].v : 0;
  }
  function peak(s: Series): number {
    return s.points.length ? Math.max(...s.points.map((p) => p.v)) : 0;
  }
</script>

<div class="rounded-lg border bg-white p-4">
  <div class="mb-2 flex items-baseline justify-between">
    <h3 class="text-sm font-medium text-gray-700">{title}</h3>
    {#if unit}
      <span class="text-xs text-gray-400">{unit}</span>
    {/if}
  </div>

  {#if hasData}
    <div class="mb-2 flex flex-wrap gap-x-6 gap-y-1">
      {#each series as s}
        <div class="flex items-center gap-2">
          <span class="inline-block h-2 w-2 rounded-full" style="background-color: {s.color}"
          ></span>
          <span class="text-xs text-gray-500">{s.label}</span>
          <span class="font-mono text-sm font-semibold text-gray-900">{format(latest(s))}</span>
          {#if peak(s) > latest(s)}
            <span class="text-xs text-gray-400">peak {format(peak(s))}</span>
          {/if}
        </div>
      {/each}
    </div>

    <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" class="h-16 w-full">
      {#each series as s}
        <path d={areaPath(s.points)} fill={s.color} fill-opacity="0.1" />
        <path
          d={linePath(s.points)}
          fill="none"
          stroke={s.color}
          stroke-width="1"
          vector-effect="non-scaling-stroke"
        />
      {/each}
    </svg>
  {:else}
    <p class="py-6 text-center text-xs text-gray-400">No samples yet</p>
  {/if}
</div>
