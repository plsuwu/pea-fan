<script lang="ts">
	import { fade } from "svelte/transition";

	let {
		src,
		alt,
		class: className = "",
		skeletonClass = "",
		element = $bindable(),
		...rest
	}: {
		src: string;
		alt: string;
		class?: string;
		skeletonClass?: string;
		element?: Element;
		[key: string]: unknown;
	} = $props();

	let loaded = $state(false);
	let errored = $state(false);
</script>

<span class="skeleton-host inline-grid place-items-center">
	{#if !loaded && !errored}
		<span
			out:fade={{ duration: 250 }}
			class="skeleton rounded-full {skeletonClass}"
			aria-hidden="true"
		></span>
	{/if}
	<img
		bind:this={element}
		{src}
		{alt}
		{...rest}
		class="{className} img-fade"
		class:visible={loaded}
		onload={() => (loaded = true)}
		onerror={() => (errored = true)}
	/>
</span>

<style>
	.skeleton-host {
		display: inline-grid;
		place-items: center;
	}

	.skeleton-host > * {
		grid-area: 1 / 1;
	}

	.skeleton {
		width: 100%;
		height: 100%;

		background: color-mix(in srgb, currentColor 1%, var(--color-background));
		overflow: hidden;
		isolation: isolate;
		position: relative;

		&::after {
			content: "";
			position: absolute;
			inset: 0;
			background: linear-gradient(
				90deg,
				transparent 0%,
				color-mix(in srgb, currentColor 4%, transparent) 5%,
				color-mix(in srgb, currentColor 14%, transparent) 40%,
				color-mix(in srgb, currentColor 4%, transparent) 95%,
				transparent 100%
			);
			background-size: 800% 600%;
			transform: translateX(-100%);
			animation: shimmer 1.5s ease-in-out infinite;
		}
	}

	@keyframes shimmer {
		to {
			transform: translateX(600%);
		}
	}

	.img-fade {
		opacity: 0;
	}

	.img-fade.visible {
		opacity: 1;
	}
</style>
