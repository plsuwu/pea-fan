<script lang="ts">
	let {
		fill = "transparent",
		stroke = "currentColor",
		size = 40,
        thickness = 1,
		disabled = false
	}: {
		fill?: "transparent" | string;
		stroke?: "currentColor" | string;
        thickness?: number;
		size?: number;
		disabled?: boolean;
	} = $props();

	let points = $derived(`0,0 ${size},0 ${size},${size} 0,${size}`);
</script>

<svg height={size} width={size} aria-hidden="true">
	<polygon
		{points}
		{fill}
		stroke={disabled
			? "color-mix(in oklab, var(--muted-foreground) 60%, transparent)"
			: stroke}
        stroke-width={thickness}
	/>
</svg>
