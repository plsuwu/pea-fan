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
</script>

<span class="skeleton-host inline-grid place-items-center">
	{#if !loaded && !errored}
		<span
			class="skeleton bg-accent-foreground {skeletonClass} img-fade"
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
		background: color-mix(in srgb, currentColor 12%, transparent);
		overflow: hidden;
		position: relative;
		isolation: isolate;
	}

	.skeleton::after {
		content: "";
		position: absolute;
		inset: 0;
		background: linear-gradient(
			90deg,
			transparent 0%,
			color-mix(in srgb, currentColor 20%, transparent) 50%,
			transparent 100%
		);
		transform: translateX(-100%);
		animation: shimmer 1.4s ease-out infinite;
		will-change: transform;
	}

	.img-fade {
		opacity: 0;
		transition: opacity 0.2s ease-out;
	}

	.img-fade.visible {
		opacity: 1;
	}
</style>
