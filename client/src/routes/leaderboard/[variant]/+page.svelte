<script lang="ts">
	import type { ChatterEntry, ChannelEntry } from "$lib/types";
	import { capitalize, mapPagedResponseToEntries } from "$lib/utils";
	import type { PageData } from "./$types";
	import Table from "$lib/components/leaderboard/table.svelte";

	let { data }: { data: PageData } = $props();
	let leaderboard = $derived(data.leaderboardData);
	let variant = $derived(
		capitalize<"Chatter" | "Channel">(data.leaderboardVariant)
	);

	let entries = $derived(mapPagedResponseToEntries(leaderboard, variant));
</script>

{#snippet Row(entry: ChatterEntry | ChannelEntry)}
	<li class="group my-2 flex flex-row items-center space-x-8">
		<img
			src={entry.image}
			alt={entry.login}
			class="rounded-full brightness-80 transition-discrete duration-100 ease-in
            group-hover:brightness-100"
		/>
		<div
			style={`color: ${entry.color.replace("#000000", "#A9A9A9")};`}
			class="text-xl font-medium brightness-25 transition-discrete duration-100
            ease-in group-hover:brightness-100"
		>
			{entry.name}
		</div>
	</li>
{/snippet}

<Table {entries} />

<!-- <div class="flex flex-col items-center"> -->
<!-- <div class="w-2/3"> -->
<!---->
<!-- </div> -->
<!-- </div> -->

<!-- <ul> -->
<!-- 	{#each entries as e} -->
<!-- 		{@const entry = e.data} -->
<!-- 		{#if variant === "Channel"} -->
<!-- 			<a href={rtUtil.getTenantHref(entry.login).href}> -->
<!-- 				{@render Row(entry)} -->
<!-- 			</a> -->
<!-- 		{:else} -->
<!-- 			{@render Row(entry)} -->
<!-- 		{/if} -->
<!-- 	{/each} -->
<!-- </ul> -->
