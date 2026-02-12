<script lang="ts">
	import type { Snippet } from "svelte";
	import { fade } from "svelte/transition";
	import { mode } from "mode-watcher";

	let {
		color = { dark: "255,255,255", light: "0,0,0" },
		opacity = 0.4,
		minSize = 0.5,
		maxSize = 3,
		count = 100,
		children
	} = $props();

	const TILE = 512;

	let dataUrl = $state<string>("");

	$effect(() => {
		if (dataUrl === "" && document) {
			dataUrl = getDataUrl();
		}
	});

	function getDataUrl() {
		const canvas = document.createElement("canvas");
		canvas.width = TILE;
		canvas.height = TILE;

		const ctx = canvas.getContext("2d")!;
		ctx.clearRect(0, 0, TILE, TILE);
		const colorOpt = mode.current === "dark" ? color.dark : color.light;

		// const imageData = ctx.createImageData(TILE, TILE);
		// const d = imageData.data;
		// const threshold = 1 - density / 100;

		for (let i = 0; i < count; i++) {
			const x = Math.random() * TILE;
			const y = Math.random() * TILE;

			const size = minSize + (maxSize - minSize) * Math.random();
			const alpha = 0.6 * Math.random();
			ctx.fillStyle = `rgba(${colorOpt},${alpha})`;

			if (size <= 1.5) {
				ctx.fillRect(
					Math.round(x),
					Math.round(y),
					Math.ceil(size),
					Math.ceil(size)
				);
			} else {
				ctx.beginPath();
				ctx.arc(x, y, size / 2, 0, Math.PI * 2);
				ctx.fill();
			}
		}

		return canvas.toDataURL("image/png");
	}
</script>

{#key dataUrl}
	<div
		transition:fade
		class="noise"
		style:background-image="url({dataUrl})"
		style:opacity
		aria-hidden="true"
	></div>
{/key}

{@render children()}

<style>
	.noise {
		position: fixed;
		inset: 0;
		z-index: -20;
		pointer-events: none;
		background-repeat: repeat;
		background-size: 200px 200px;
	}
</style>
