<script lang="ts">
	import type { PageData } from "./$types";
	import Tenant from "$lib/components/tenant/tenant.svelte";
	import { intoParentEntry, type UntypedEntry } from "$lib/utils/index.js";
	import { page } from "$app/state";
	import Stats from "$lib/components/tenant/stats/stats.svelte";

	let { data } = $props();
	let { channel, scoreWindows, leaderboard, modePreference }: PageData =
		$derived(data);

	let channelData: UntypedEntry<UntypedEntry> | null = $derived.by(() => {
		if (!channel || !leaderboard) return null;

		const [channelItems] = leaderboard.items;
		const scores = channelItems.scores.map((entry) => intoParentEntry(entry));

		return { ...channelItems, scores };
	});

	let paginationData = $derived.by(() => {
		if (!leaderboard) return null;
		const { page, total_items, total_pages, page_size } = leaderboard;
		return {
			currentPage: page,
			totalItems: total_items,
			itemsPerPage: page_size,
			totalPages: total_pages,
		};
	});
</script>

<div class="w-[93%] self-center xl:w-[90%]">
	{#if channel && channelData && paginationData && scoreWindows}
		<div class="flex flex-col lg:flex-row xl:justify-center">
			<div class="w-full px-2 md:min-w-[320px] lg:pr-6 lg:pl-2 xl:w-[35%]">
				<Stats channelLogin={channel} {channelData} {scoreWindows} />
			</div>
			<Tenant channelLogin={channel} {channelData} {paginationData} />
		</div>
	{/if}
</div>
