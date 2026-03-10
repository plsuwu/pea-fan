<script lang="ts">
	import type { Snippet } from "svelte";

	let {
		href,
		handleMouseEvent,
		children,
	}: {
		href?: string;
		handleMouseEvent: (event: MouseEvent) => void;
		children: Snippet;
	} = $props();
</script>

{#if href}
	<a
		{href}
		onfocus={() => {}}
		class="flex h-full w-full flex-row items-center justify-between space-x-4
            brightness-90 transition-all duration-300 ease-in-out hover:brightness-100"
		onmouseover={(event) => handleMouseEvent(event)}
		onmouseleave={(event) => handleMouseEvent(event)}
	>
		{@render children()}
	</a>
{:else}
	<div
		role="none"
		onfocus={() => {}}
		class="flex h-full w-full cursor-default flex-row items-center justify-between
            space-x-4 brightness-90 transition-all duration-300 ease-in-out
            hover:cursor-default hover:brightness-100"
		onmouseover={(event) => handleMouseEvent(event)}
		onmouseleave={(event) => handleMouseEvent(event)}
	>
		{@render children()}
	</div>
{/if}
