<script lang="ts">
	import { mode } from "mode-watcher";
	import RowLayout from "./row/layout.svelte";
	import type { UntypedEntry } from "$lib/utils";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import { navigating } from "$app/state";
	import { fade } from "svelte/transition";

	let {
		entries,
		variant,
	}: {
		entries: (UntypedEntry & { isLive: boolean })[];
		variant: "Chatter" | "Channel";
	} = $props();
	let currentMode = $derived(mode.current);
</script>

{#snippet Loader()}
	<div
		in:fade={{ delay: 0, duration: 150 }}
		out:fade={{ delay: 0, duration: 150 }}
		class="absolute z-30 flex h-full min-h-screen w-full items-center justify-center bg-background/75"
	>
		<Spinner />
	</div>
{/snippet}

<div class="flex flex-col space-y-1">
	<div class="relative h-full w-full">
		{#if navigating.to}
			{@render Loader()}
		{/if}
	</div>
	<div
		in:fade={{ delay: 0, duration: 0 }}
		out:fade={{ delay: 100, duration: 0 }}
	>
		{#each entries as entry}
			<RowLayout {entry} {variant} mode={currentMode} showScoreIcons={true} />
		{/each}
	</div>
</div>
