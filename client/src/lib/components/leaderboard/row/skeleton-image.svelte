<script lang="ts">
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
	let inView = $state(true);
	let host = $state<HTMLElement>();

	$effect(() => {
		const el = host;
		if (!el) return;

		const io = new IntersectionObserver(
			(entries) => {
				inView = entries[0].isIntersecting;
			},
			{ rootMargin: "100px" }
		);
		io.observe(el);
		return () => io.disconnect();
	});

	let animate = $derived(inView && !loaded && !errored);
</script>

<span bind:this={host} class="skeleton-host inline-grid place-items-center">
	<span
		class="skeleton rounded-full {skeletonClass}"
		class:is-hidden={loaded || errored}
		class:animate
		aria-hidden="true"
	></span>

	<img
		loading="lazy"
		decoding="async"
		bind:this={element}
		{src}
		{alt}
		{...rest}
		class="{className} img-fade"
		class:visible={loaded && !errored}
		onload={() => (loaded = true)}
		onerror={() => (errored = true)}
	/>

	{#if errored}
		<div class={className}></div>
	{/if}
</span>

<style>
	.skeleton-host {
		display: inline-grid;
		place-items: center;
		contain: layout style paint;
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
		opacity: 1;
		transition: opacity 250ms ease-out;
	}

	.skeleton.animate::after {
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
		will-change: transform;
	}

	.skeleton.is-hidden {
		opacity: 0;
		pointer-events: none;
	}

	.skeleton.is-hidden::after {
		animation-play-state: paused;
	}

	@keyframes shimmer {
		to {
			transform: translateX(600%);
		}
	}

	.img-fade {
		opacity: 0;
		transition: opacity 250ms ease-out;
	}

	.img-fade.visible {
		opacity: 1;
	}
</style>
