<script lang="ts">
	import RowLayout from "$lib/components/leaderboard/row/layout.svelte";
	import { mode } from "mode-watcher";
	import { readableColor, type UntypedEntry } from "$lib/utils";
	import Stats from "./stats/stats.svelte";
	import Pagination from "../leaderboard/filtering/pagination.svelte";
	import { page } from "$app/state";

	let {
		channelLogin,
		channelData,
		paginationData,
	}: {
		channelLogin: string;
		channelData: UntypedEntry<UntypedEntry>;
		paginationData: {
			currentPage: number;
			totalItems: number;
			itemsPerPage: number;
			totalPages: number;
		};
	} = $props();

	let { currentPage, totalItems, totalPages, itemsPerPage } =
		$derived(paginationData);

	let displayName = $derived(channelData.name);
	let scoreEntries = $derived(channelData.scores);

	let currentUrl = $derived.by(() => {
		const currentUrlBase = page.url;
		const { score_page } = Object.fromEntries(page.url.searchParams);
		$inspect(score_page);

		return currentUrlBase;
	});
</script>

<div class="my-10 lg:my-4 xl:mx-14"></div>
<div
	class="w-full self-center lg:max-w-[610px] lg:min-w-[610px] lg:self-start xl:min-w-[350px]"
>
	{#if scoreEntries && scoreEntries.length > 0}
		<div class="-mt-10 mb-22 text-center text-2xl">
			<span class="font-bold">{displayName}</span>'s top pissers
		</div>
		<div
			class="flex shrink-0 flex-col space-y-2 lg:max-w-[850px] lg:min-w-[500px]"
		>
			<Pagination
				pageNumber={paginationData.currentPage}
				{currentUrl}
				{totalItems}
				{itemsPerPage}
				{totalPages}
				pageParam={"score_page"}
			/>
			<div class="my-8"></div>
			{#each scoreEntries as entry}
				<RowLayout
					{entry}
					variant={"Chatter"}
					showScoreIcons={false}
					mode={mode.current}
				/>
			{/each}
		</div>
	{:else}
		<div class="relative flex min-h-[70vh] items-center justify-center">
			piss has never been mentioned here
		</div>
	{/if}
</div>
