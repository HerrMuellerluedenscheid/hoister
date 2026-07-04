<script lang="ts">
	/**
	 * The four resource graphs (CPU, memory, network, disk I/O) for one service's
	 * metric time series. Shared by the per-container detail page and the
	 * aggregate Resources page so both derive the series identically.
	 */
	import TimeSeriesChart from './TimeSeriesChart.svelte';
	import type { MetricPointResponse } from '../../bindings/MetricPointResponse';

	let { points }: { points: MetricPointResponse[] } = $props();

	const cpuSeries = $derived(points.map((p) => ({ t: Date.parse(p.recorded_at), v: p.cpu_pct })));
	const memSeries = $derived(points.map((p) => ({ t: Date.parse(p.recorded_at), v: p.mem_bytes })));
	// Cap the memory chart at the container's limit when one is set, so the
	// graph reflects headroom rather than auto-scaling to the peak.
	const memLimit = $derived(points.length > 0 ? points[points.length - 1].mem_limit_bytes : 0);

	// Network and disk are cumulative byte counters since container start, so a
	// raw plot just ramps upward. Differentiate against the previous sample to
	// recover throughput in bytes/second; the first sample has no predecessor
	// and a counter that goes backwards (container restart) is clamped to 0.
	function rateSeries(field: (p: MetricPointResponse) => number): { t: number; v: number }[] {
		const out: { t: number; v: number }[] = [];
		for (let i = 1; i < points.length; i++) {
			const t = Date.parse(points[i].recorded_at);
			const seconds = (t - Date.parse(points[i - 1].recorded_at)) / 1000;
			if (seconds <= 0) continue;
			const delta = field(points[i]) - field(points[i - 1]);
			out.push({ t, v: delta > 0 ? delta / seconds : 0 });
		}
		return out;
	}

	const netSeries = $derived([
		{ label: 'RX', color: '#34d399', points: rateSeries((p) => p.net_rx_bytes) },
		{ label: 'TX', color: '#fbbf24', points: rateSeries((p) => p.net_tx_bytes) }
	]);
	const diskSeries = $derived([
		{ label: 'Read', color: '#38bdf8', points: rateSeries((p) => p.disk_read_bytes) },
		{ label: 'Write', color: '#f472b6', points: rateSeries((p) => p.disk_write_bytes) }
	]);

	function formatBytes(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
		return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function formatRate(bytesPerSecond: number): string {
		return `${formatBytes(bytesPerSecond)}/s`;
	}
</script>

<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
	<TimeSeriesChart
		points={cpuSeries}
		label="CPU"
		color="#818cf8"
		formatValue={(v) => `${v.toFixed(1)}%`}
	/>
	<TimeSeriesChart
		points={memSeries}
		label="Memory"
		color="#34d399"
		formatValue={formatBytes}
		max={memLimit > 0 ? memLimit : undefined}
	/>
	<TimeSeriesChart series={netSeries} label="Network" formatValue={formatRate} />
	<TimeSeriesChart series={diskSeries} label="Disk I/O" formatValue={formatRate} />
</div>
