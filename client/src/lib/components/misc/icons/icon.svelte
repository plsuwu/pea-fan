<script lang="ts">
	import "iconify-icon";
	import { onMount } from "svelte";
    import { loadedIconCache } from "./load-cache.svelte";

	let {
		icon,
		color = "currentColor",
		size = 20,
	}: { icon: string; color: string; size?: number } = $props();

	let iconEl = $state<Element | null>(null);
	let loaded = $derived(loadedIconCache.has(icon));

	onMount(() => {
		if (loaded || !iconEl) return;

		const markLoaded = () => {
			loadedIconCache.add(icon);
			loaded = true;
			return true;
		};

		const checkLoaded = () =>
			iconEl?.shadowRoot?.querySelector("svg") ? markLoaded() : false;

		if (checkLoaded()) return;

		let svgObserver: MutationObserver | undefined;
		const observeShadowRoot = () => {
			if (!iconEl?.shadowRoot) return false;
			svgObserver = new MutationObserver(() => {
				if (checkLoaded()) svgObserver?.disconnect();
			});
			svgObserver.observe(iconEl.shadowRoot, {
				childList: true,
				subtree: true,
			});

			// race check on-attach
			checkLoaded();
			return true;
		};

		if (!observeShadowRoot()) {
			const interval = setInterval(() => {
				if (observeShadowRoot()) clearInterval(interval);
			}, 20);

			return () => {
				clearInterval(interval);
				svgObserver?.disconnect();
			};
		}

		return () => svgObserver?.disconnect();
	});
</script>

<div class="icon-host" style:width="{size}px" style:height="{size}px">
	{#if !loaded}
		<div class="skeleton" style:width="{size}px" style:height="{size}px"></div>
	{/if}
	<div class="items-center justify-center">
		<iconify-icon
			bind:this={iconEl}
			{icon}
			width={size}
			height={size}
			style:color
			class:visible={loaded}
		></iconify-icon>
	</div>
</div>

<style>
	.icon-host {
		display: inline-grid;
		place-items: center;
		align-items: center;
		position: relative;
	}

	/* .icon-host > * { */
	/* 	grid-area: 1 / 1; */
	/* } */

	.skeleton {
		border-radius: 8px;
		background: color-mix(in srgb, currentColor 15%, transparent);
		animation: pulse 1.2s ease-in-out infinite;
	}

	iconify-icon {
		display: inline-block;
		opacity: 0;
		transition: opacity 0.25s ease;
	}

	iconify-icon.visible {
		opacity: 1;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 0.4;
		}
		50% {
			opacity: 0.9;
		}
	}
</style>
