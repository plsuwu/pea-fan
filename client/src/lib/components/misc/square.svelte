<script lang="ts">
	let {
		size = 40,
		disabled = false
	}: {
		size?: number;
		disabled?: boolean;
	} = $props();

	let points = $derived(`0,0 ${size},0 ${size},${size} 0,${size}`);
</script>

<svg height={size} width={size} aria-hidden="true">
	<polygon
		{points}
		fill="transparent"
		stroke={disabled
			? "color-mix(in oklab, var(--muted-foreground) 60%, transparent)"
			: "currentColor"}
	/>
</svg>
