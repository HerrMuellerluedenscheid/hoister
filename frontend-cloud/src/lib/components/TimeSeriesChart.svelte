<script lang="ts">
	/**
	 * Dependency-free SVG line/area chart for a single metric over time.
	 * The parent maps domain points to `{ t, v }` (t = epoch ms, v = value) and
	 * supplies a value formatter. Renders responsively via a fixed viewBox so
	 * we avoid pulling in a charting library.
	 */
	interface Point {
		t: number;
		v: number;
	}

	let {
		points,
		label,
		color = '#818cf8',
		formatValue = (v: number) => v.toFixed(1),
		// Optional fixed upper bound for the y-axis (e.g. memory limit). When
		// omitted the axis auto-scales to the data's max.
		max = undefined
	}: {
		points: Point[];
		label: string;
		color?: string;
		formatValue?: (v: number) => string;
		max?: number | undefined;
	} = $props();

	// viewBox coordinate space. Scales to the container width via CSS.
	const W = 600;
	const H = 180;
	const PAD = { top: 12, right: 12, bottom: 24, left: 48 };
	const plotW = W - PAD.left - PAD.right;
	const plotH = H - PAD.top - PAD.bottom;

	const sorted = $derived([...points].sort((a, b) => a.t - b.t));

	const stats = $derived.by(() => {
		if (sorted.length === 0) return null;
		const vs = sorted.map((p) => p.v);
		const tMin = sorted[0].t;
		const tMax = sorted[sorted.length - 1].t;
		const vMaxData = Math.max(...vs);
		const vMax = Math.max(max ?? 0, vMaxData, 1e-9);
		return {
			tMin,
			tMax,
			tSpan: Math.max(tMax - tMin, 1),
			vMax,
			last: vs[vs.length - 1],
			min: Math.min(...vs),
			peak: vMaxData
		};
	});

	function x(t: number): number {
		const s = stats!;
		return PAD.left + ((t - s.tMin) / s.tSpan) * plotW;
	}
	function y(v: number): number {
		const s = stats!;
		return PAD.top + plotH - (v / s.vMax) * plotH;
	}

	const linePath = $derived.by(() => {
		if (!stats || sorted.length === 0) return '';
		return sorted.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.t)},${y(p.v)}`).join(' ');
	});

	const areaPath = $derived.by(() => {
		if (!stats || sorted.length === 0) return '';
		const top = sorted.map((p) => `L${x(p.t)},${y(p.v)}`).join(' ');
		return `M${x(sorted[0].t)},${PAD.top + plotH} ${top} L${x(sorted[sorted.length - 1].t)},${PAD.top + plotH} Z`;
	});

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

	// Hover state.
	let hover = $state<{ x: number; y: number; p: Point } | null>(null);

	function onMove(e: PointerEvent) {
		if (!stats || sorted.length === 0) return;
		const svg = e.currentTarget as SVGSVGElement;
		const rect = svg.getBoundingClientRect();
		// Map client x into viewBox x, then into the data domain.
		const vbX = ((e.clientX - rect.left) / rect.width) * W;
		const frac = Math.min(Math.max((vbX - PAD.left) / plotW, 0), 1);
		const targetT = stats.tMin + frac * stats.tSpan;
		// Nearest point by time.
		let nearest = sorted[0];
		for (const p of sorted) {
			if (Math.abs(p.t - targetT) < Math.abs(nearest.t - targetT)) nearest = p;
		}
		hover = { x: x(nearest.t), y: y(nearest.v), p: nearest };
	}
</script>

<div class="rounded-xl border border-zinc-800 bg-zinc-900 p-4">
	<div class="mb-2 flex items-baseline justify-between">
		<h3 class="text-sm font-medium text-zinc-300">{label}</h3>
		{#if stats}
			<span class="font-mono text-xs text-zinc-400">
				now {formatValue(stats.last)} · peak {formatValue(stats.peak)}
			</span>
		{/if}
	</div>

	{#if !stats}
		<div class="flex h-40 items-center justify-center text-sm text-zinc-600">No data</div>
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
					stroke="#27272a"
					stroke-width="1"
				/>
				<text x={PAD.left - 6} y={tick.py + 3} text-anchor="end" class="fill-zinc-500 text-[10px]">
					{formatValue(tick.v)}
				</text>
			{/each}

			<!-- x labels: first + last sample time -->
			<text x={PAD.left} y={H - 6} text-anchor="start" class="fill-zinc-500 text-[10px]">
				{fmtTime(stats.tMin)}
			</text>
			<text x={W - PAD.right} y={H - 6} text-anchor="end" class="fill-zinc-500 text-[10px]">
				{fmtTime(stats.tMax)}
			</text>

			<path d={areaPath} fill={color} fill-opacity="0.12" />
			<path d={linePath} fill="none" stroke={color} stroke-width="1.5" />

			{#if hover}
				<line
					x1={hover.x}
					y1={PAD.top}
					x2={hover.x}
					y2={PAD.top + plotH}
					stroke="#52525b"
					stroke-width="1"
					stroke-dasharray="3 3"
				/>
				<circle cx={hover.x} cy={hover.y} r="3" fill={color} />
			{/if}
		</svg>

		{#if hover}
			<p class="mt-1 text-center font-mono text-xs text-zinc-400">
				{formatValue(hover.p.v)} · {fmtTime(hover.p.t)}
			</p>
		{/if}
	{/if}
</div>
