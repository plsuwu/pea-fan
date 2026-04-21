<script lang="ts">
	import {
		capitalize,
		intoUntypedEntry,
		mapPagedResponseToEntries,
	} from "$lib/utils";
	import type { PageData } from "./$types";
	import Table from "$lib/components/leaderboard/table.svelte";
	import Pagination from "$lib/components/leaderboard/filtering/pagination.svelte";
	import { page } from "$app/state";
	import { Checkbox } from "$lib/shadcn-components/ui/checkbox";
	import { Label } from "$lib/shadcn-components/ui/label";

	let { data }: { data: PageData } = $props();
	let leaderboard = $derived(data.leaderboardData);
	let isLive = $derived(data.liveBroadcasters);

	let variant = $derived(
		capitalize<"Chatter" | "Channel">(data.leaderboardVariant)
	);

	let entries = $derived.by(() => {
		const paginated = mapPagedResponseToEntries(leaderboard, variant);
		return paginated.map((entry) => {
			const untypedEntry = intoUntypedEntry(entry);
			const live: boolean = isLive.some(
				(br: any & { id: string }) => br.id === untypedEntry.id
			);

			return { ...untypedEntry, isLive: live };
		});
	});

	let currentUrl = $derived(page.url);

	let { pageNumber, totalPages, totalItems, itemsPerPage } = $derived.by(() => {
		const { page, total_pages, total_items, page_size } = data.leaderboardData;
		return {
			pageNumber: page,
			totalPages: total_pages,
			totalItems: total_items,
			itemsPerPage: page_size,
		};
	});

	let onlyShowLive = $state(false);
</script>

<svelte:head>
	<meta
		name="description"
		content="leaderboard ranking a handful of twitch broadcasters by the number of times a chatter's messages have contained the word piss."
	/>
</svelte:head>

<div class="flex w-full flex-col">
	<div
		class="flex w-[60%] flex-col self-center pb-8 transition-all duration-100 ease-in
    lg:w-[650px]"
	>
		<div
			class="mx-0 mt-4 flex flex-col items-center justify-center space-x-4 text-center
            text-xl font-bold -tracking-wider text-foreground lowercase md:text-start md:text-2xl
            lg:mx-8 lg:flex-row lg:text-[40px]"
		>
			<div>piss leaderboard ({variant}s)</div>
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
		{#if variant === "Channel"}
			<div class="mb-4 flex flex-row justify-end space-x-2 px-4">
				<Label for="only-live" class="-tracking-wider text-muted-foreground/60"
					>show live channels only</Label
				>
				<Checkbox id="only-live" bind:checked={onlyShowLive} />
			</div>
		{/if}
		<Table {entries} {variant} {onlyShowLive} />
	</div>
</div>
