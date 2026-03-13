<script lang="ts">
	import type { ScoreWindows } from "$lib/types";
	import { readableColor, type UntypedEntry } from "$lib/utils";
	import type { Snippet } from "svelte";
	import ExternalLinks from "./external.svelte";
	import Nameplate from "./nameplate.svelte";
	import Periodic from "./periodic.svelte";
	let {
		channelLogin,
		channelData,
		scoreWindows,
		children,
	}: {
		channelLogin: string;
		channelData: UntypedEntry<UntypedEntry>;
		scoreWindows: ScoreWindows;
		children: Snippet;
	} = $props();

	let {
		yesterday,
		prev_week,
		prev_month,
		prev_year,
		last_7_days,
		last_30_days,
	} = $derived(scoreWindows);
</script>

{#snippet StatLine(title: string, stat: string | number)}
	<div
		class="flex items-center justify-between space-x-1 text-[14px] md:text-[18px]"
	>
		<div class="font-semibold text-accent-foreground/55">{title}:</div>
		<div>{stat}</div>
	</div>
{/snippet}

<!-- {#snippet WindowedStat(title: string, stat: number)} -->
<!-- 	{#if stat > 0} -->
<!-- 		{@render StatLine(title, stat.toLocaleString())} -->
<!-- 	{/if} -->
<!-- {/snippet} -->

<div class="w-full lg:fixed">
	<div class="flex w-full flex-col">
		<Nameplate
			channel={channelData.name}
			color={readableColor(channelData.color)}
			image={channelData.image}
		/>
		{@render children()}
		<div
			class="mt-1 mb-4 border-b border-b-muted lg:w-[30%] lg:max-w-[500px] xl:w-[35%] xl:max-w-[600px]"
		></div>
	</div>
	<div
		class="w-full space-y-1 lg:w-[30%] lg:max-w-[500px] xl:w-[35%] xl:max-w-[600px]"
	>
		{@render StatLine("channel rank", channelData.ranking)}
		{@render StatLine(
			"total piss mentions",
			channelData.total.toLocaleString()
		)}
		{@render StatLine(
			"unique piss mentioners",
			channelData.totalScores.toLocaleString()
		)}

		<div class="mb-12 border-b border-b-muted pt-1"></div>
		<Periodic
			{yesterday}
			prevWeek={prev_week}
			prevMonth={prev_month}
			prevYear={prev_year}
			last7Days={last_7_days}
			last30Days={last_30_days}
		/>
	</div>
</div>
