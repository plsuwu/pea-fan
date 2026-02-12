<script lang="ts">
	import { ChevronRight, ChevronLeft } from "@lucide/svelte";
	import PageButton from "./page-button.svelte";

	let {
		pageNumber,
		totalPages,
		currentUrl,
		totalItems,
		itemsPerPage,
		variant
	}: {
		pageNumber: number;
		totalPages: number;
		currentUrl: URL;
		totalItems: number;
		itemsPerPage: number;
		variant: string;
	} = $props();

	let currentRanks = $derived({
		// low is 0-indexed
		low: itemsPerPage * pageNumber - itemsPerPage + 1,
		high: itemsPerPage * pageNumber
	});

	function withPage(page: number): string {
		const url = new URL(currentUrl.href);
		url.searchParams.set("page", String(page));
		return url.href;
	}

	let hasPrev = $derived(pageNumber > 1);
	let hasNext = $derived(pageNumber < totalPages);
</script>

<div class="mb-4 flex w-full flex-col justify-center">
	<div class="mb-4 flex justify-center">
		<div class="flex w-4/5 flex-row justify-between">
			<PageButton
				icon={ChevronLeft}
				href={withPage(pageNumber - 1)}
				disabled={!hasPrev}
				direction="backward"
				class="mr-0.5"
			/>
			<PageButton
				icon={ChevronRight}
				href={withPage(pageNumber + 1)}
				disabled={!hasNext}
				direction="forward"
				class="ml-0.5"
			/>
		</div>
	</div>
	<div class="text-center text-muted-foreground">
		<div class="text-sm">page {pageNumber} of {totalPages}</div>
		<div class="text-xs italic">
			rank {currentRanks.low} to {currentRanks.high} (of {totalItems} total)
		</div>
	</div>
</div>
