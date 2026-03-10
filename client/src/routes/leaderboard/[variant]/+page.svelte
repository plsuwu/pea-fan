<script lang="ts">
	import {
		capitalize,
		intoUntypedEntry,
		mapPagedResponseToEntries
	} from "$lib/utils";
	import type { PageData } from "./$types";
	import Table from "$lib/components/leaderboard/table.svelte";
	import Pagination from "$lib/components/leaderboard/filtering/pagination.svelte";
	import { page } from "$app/state";

	let { data }: { data: PageData } = $props();
	let leaderboard = $derived(data.leaderboardData);
    console.log(leaderboard);

	let variant = $derived(
		capitalize<"Chatter" | "Channel">(data.leaderboardVariant)
	);

	let entries = $derived.by(() => {
		const paginated = mapPagedResponseToEntries(leaderboard, variant);
		return paginated.map((entry) => intoUntypedEntry(entry));
	});

	let currentUrl = $derived(page.url);

	let { pageNumber, totalPages, totalItems, itemsPerPage } = $derived.by(() => {
		const { page, total_pages, total_items, page_size } = data.leaderboardData;
		return {
			pageNumber: page,
			totalPages: total_pages,
			totalItems: total_items,
			itemsPerPage: page_size
		};
	});
</script>

<div
	class="flex w-[60%] flex-col self-center pb-8 transition-all duration-100 ease-in
    lg:w-[650px]"
>
	<div
		class="mx-0 mt-4 flex flex-col items-center justify-center space-x-4 text-[15px] font-semibold
        -tracking-wider text-foreground uppercase md:text-2xl lg:mx-8 lg:flex-row lg:text-4xl"
	>
		<div>piss leaderboard - {variant}s</div>
	</div>
	<div class="w-full border-b border-b-foreground pb-4"></div>
</div>

<div class="w-11/12 self-center md:w-[750px]">
	{#if variant === "Chatter"}
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
	{/if}
	<Table {entries} {variant} />
</div>
