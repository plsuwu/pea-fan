<script lang="ts">
	import { capitalize, mapPagedResponseToEntries } from "$lib/utils";
	import type { PageData } from "./$types";
	import Table from "$lib/components/leaderboard/table.svelte";
	import Pagination from "$lib/components/leaderboard/filtering/pagination.svelte";
	import { page } from "$app/state";

	let { data }: { data: PageData } = $props();
	let leaderboard = $derived(data.leaderboardData);
	let variant = $derived(
		capitalize<"Chatter" | "Channel">(data.leaderboardVariant)
	);

	let entries = $derived(mapPagedResponseToEntries(leaderboard, variant));

	let pageNumber = $derived(data.leaderboardData.page);
	let totalPages = $derived(data.leaderboardData.total_pages);
	let currentUrl = $state(page.url);
	let totalItems = $derived(data.leaderboardData.total_items);
	let itemsPerPage = $derived(data.leaderboardData.page_size);
</script>

<div class="flex w-[40%] flex-col self-center pb-8 xl:w-[25%]">
	<div class="mt-4 flex flex-col">
		<span class="text-2xl font-bold text-foreground 2xl:text-4xl"
			>PISS LEADERBOARD</span
		>
		<span
			class="mt-1 text-base font-semibold text-accent-foreground/50 uppercase italic lg:text-lg"
			>({variant}s)</span
		>
	</div>
	<div class="w-full border-b border-b-foreground pb-4"></div>
</div>
<div class="w-11/12 self-center xl:w-[750px]">
	<div class="mt-8 mb-8 flex w-full items-center justify-between">
		<Pagination
			{pageNumber}
			{currentUrl}
			{totalPages}
			{totalItems}
			{itemsPerPage}
			{variant}
		/>
	</div>
	<Table {entries} />
</div>
