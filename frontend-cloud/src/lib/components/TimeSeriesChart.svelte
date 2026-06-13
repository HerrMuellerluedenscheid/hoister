<script lang="ts">
	/**
	 * Dependency-free SVG line/area chart for one or more metric series sharing a
	 * y-scale (e.g. network rx + tx, disk read + write). The parent maps domain
	 * points to `{ t, v }` (t = epoch ms, v = value) and supplies a value
	 * formatter. Renders responsively via a fixed viewBox so we avoid pulling in
	 * a charting library.
	 */
	interface Point {
		t: number;
		v: number;
	}
	interface Series {
		label: string;
		color: string;
		points: Point[];
	}

	let {
		points,
		series,
		label,
		color = 'var(--color-brand-accent)',
		formatValue = (v: number) => v.toFixed(1),
		// Optional fixed upper bound for the y-axis (e.g. memory limit). When
		// omitted the axis auto-scales to the data's max.
		max = undefined
	}: {
		// Single-series shorthand; `series` takes precedence when provided.
		points?: Point[];
		series?: Series[];
		label: string;
		color?: string;
		formatValue?: (v: number) => string;
		max?: number | undefined;
	} = $props();

	// Normalize to a list of series, each sorted by time. The single-series
	// `points`/`color` props collapse into a one-element list.
	const resolved = $derived<Series[]>(
		series && series.length > 0
			? series.map((s) => ({ ...s, points: [...s.points].sort((a, b) => a.t - b.t) }))
			: [{ label, color, points: [...(points ?? [])].sort((a, b) => a.t - b.t) }]
	);

	// A legend only earns its space when there's more than one line.
	const showLegend = $derived(resolved.length > 1);

	// viewBox coordinate space. Scales to the container width via CSS.
	const W = 600;
	const H = 180;
	const PAD = { top: 12, right: 12, bottom: 24, left: 48 };
	const plotW = W - PAD.left - PAD.right;
	const plotH = H - PAD.top - PAD.bottom;

	const allPoints = $derived(resolved.flatMap((s) => s.points));

	const stats = $derived.by(() => {
		if (allPoints.length === 0) return null;
		const ts = allPoints.map((p) => p.t);
		const vs = allPoints.map((p) => p.v);
		const tMin = Math.min(...ts);
		const tMax = Math.max(...ts);
		const vMaxData = Math.max(...vs);
		// Share one upper bound across series so the lines are comparable.
		const vMax = Math.max(max ?? 0, vMaxData, 1e-9);
		return { tMin, tMax, tSpan: Math.max(tMax - tMin, 1), vMax };
	});

	function x(t: number): number {
		const s = stats!;
		return PAD.left + ((t - s.tMin) / s.tSpan) * plotW;
	}
	function y(v: number): number {
		const s = stats!;
		return PAD.top + plotH - (v / s.vMax) * plotH;
	}

	function linePath(pts: Point[]): string {
		if (!stats || pts.length === 0) return '';
		return pts.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.t)},${y(p.v)}`).join(' ');
	}

	function areaPath(pts: Point[]): string {
		if (!stats || pts.length === 0) return '';
		const top = pts.map((p) => `L${x(p.t)},${y(p.v)}`).join(' ');
		return `M${x(pts[0].t)},${PAD.top + plotH} ${top} L${x(pts[pts.length - 1].t)},${PAD.top + plotH} Z`;
	}

	function last(pts: Point[]): number {
		return pts.length ? pts[pts.length - 1].v : 0;
	}
	function peak(pts: Point[]): number {
		return pts.length ? Math.max(...pts.map((p) => p.v)) : 0;
	}

	// Three horizontal gridlines at 0, 50%, 100% of the y-axis.
	const yTicks = $derived.by(() => {
		if (!stats) return [];
		return [0, 0.5, 1].map((f) => ({ v: stats.vMax * f, py: y(stats.vMax * f) }));
	});

	const fmtTime = (t: number) =>
		new Date(t).toLocaleString(undefined, {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});

	// Hover state: a shared vertical cursor plus the nearest sample per series.
	let hover = $state<{
		x: number;
		t: number;
		items: { color: string; label: string; py: number; v: number }[];
	} | null>(null);

	function nearestInSeries(pts: Point[], targetT: number): Point | null {
		if (pts.length === 0) return null;
		let n = pts[0];
		for (const p of pts) if (Math.abs(p.t - targetT) < Math.abs(n.t - targetT)) n = p;
		return n;
	}

	function onMove(e: PointerEvent) {
		if (!stats || allPoints.length === 0) return;
		const svg = e.currentTarget as SVGSVGElement;
		const rect = svg.getBoundingClientRect();
		// Map client x into viewBox x, then into the data domain.
		const vbX = ((e.clientX - rect.left) / rect.width) * W;
		const frac = Math.min(Math.max((vbX - PAD.left) / plotW, 0), 1);
		const targetT = stats.tMin + frac * stats.tSpan;
		// Snap the cursor to the nearest actual sample time for a crisp readout.
		let snapped = targetT;
		let bestDiff = Infinity;
		for (const p of allPoints) {
			const d = Math.abs(p.t - targetT);
			if (d < bestDiff) {
				bestDiff = d;
				snapped = p.t;
			}
		}
		const items = resolved
			.map((s) => {
				const p = nearestInSeries(s.points, snapped);
				return p ? { color: s.color, label: s.label, py: y(p.v), v: p.v } : null;
			})
			.filter((i): i is NonNullable<typeof i> => i !== null);
		hover = items.length ? { x: x(snapped), t: snapped, items } : null;
	}

	const tooltip = $derived(
		hover
			? hover.items
					.map((it) => (showLegend ? `${it.label} ` : '') + formatValue(it.v))
					.join(' · ') + ` · ${fmtTime(hover.t)}`
			: ''
	);
</script>

<div class="rounded-xl border border-line bg-card p-4">
	<div class="mb-2 flex items-baseline justify-between gap-3">
		<h3 class="text-sm font-medium text-ink-secondary">{label}</h3>
		{#if stats && !showLegend}
			<span class="font-mono text-xs text-ink-muted">
				now {formatValue(last(resolved[0].points))} · peak {formatValue(peak(resolved[0].points))}
			</span>
		{/if}
	</div>

	{#if stats && showLegend}
		<div class="mb-2 flex flex-wrap gap-x-4 gap-y-1">
			{#each resolved as s}
				<div class="flex items-center gap-1.5">
					<span class="inline-block h-2 w-2 rounded-full" style="background-color: {s.color}"
					></span>
					<span class="text-xs text-ink-muted">{s.label}</span>
					<span class="font-mono text-xs text-ink-secondary">{formatValue(last(s.points))}</span>
				</div>
			{/each}
		</div>
	{/if}

	{#if !stats}
		<div class="flex h-40 items-center justify-center text-sm text-ink-ghost">No data</div>
	{:else}
		<svg
			viewBox="0 0 {W} {H}"
			class="w-full"
			role="img"
			aria-label={label}
			onpointermove={onMove}
			onpointerleave={() => (hover = null)}
		>
			<!-- gridlines + y labels -->
			{#each yTicks as tick}
				<line
					x1={PAD.left}
					y1={tick.py}
					x2={W - PAD.right}
					y2={tick.py}
					stroke="var(--color-line)"
					stroke-width="1"
				/>
				<text x={PAD.left - 6} y={tick.py + 3} text-anchor="end" class="fill-ink-faint text-[10px]">
					{formatValue(tick.v)}
				</text>
			{/each}

			<!-- x labels: first + last sample time -->
			<text x={PAD.left} y={H - 6} text-anchor="start" class="fill-ink-faint text-[10px]">
				{fmtTime(stats.tMin)}
			</text>
			<text x={W - PAD.right} y={H - 6} text-anchor="end" class="fill-ink-faint text-[10px]">
				{fmtTime(stats.tMax)}
			</text>

			{#each resolved as s}
				<path d={areaPath(s.points)} fill={s.color} fill-opacity={showLegend ? 0.06 : 0.12} />
				<path d={linePath(s.points)} fill="none" stroke={s.color} stroke-width="1.5" />
			{/each}

			{#if hover}
				<line
					x1={hover.x}
					y1={PAD.top}
					x2={hover.x}
					y2={PAD.top + plotH}
					stroke="var(--color-line-subtle)"
					stroke-width="1"
					stroke-dasharray="3 3"
				/>
				{#each hover.items as it}
					<circle cx={hover.x} cy={it.py} r="3" fill={it.color} />
				{/each}
			{/if}
		</svg>

		{#if hover}
			<p class="mt-1 text-center font-mono text-xs text-ink-muted">{tooltip}</p>
		{/if}
	{/if}
</div>
