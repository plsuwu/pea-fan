<script lang="ts">
	import { mode } from "mode-watcher";
	import RowLayout from "./row/layout.svelte";
	import type { UntypedEntry } from "$lib/utils";

	let {
		entries,
		variant,
	}: { entries: UntypedEntry[]; variant: "Chatter" | "Channel" } = $props();
	let currentMode = $derived(mode.current);
</script>

<div class="flex flex-col space-y-1">
	{#each entries as entry}
		<RowLayout {entry} {variant} mode={currentMode} showScoreIcons={true} />
	{/each}
</div>
