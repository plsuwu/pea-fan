<script lang="ts">
	import { ChevronRight, ChevronLeft } from "@lucide/svelte";
	import PageButton from "./page-button.svelte";
	import { goto } from "$app/navigation";
	import { navigating } from "$app/state";
	import { onMount } from "svelte";

	let {
		pageNumber,
		totalPages,
		currentUrl,
		totalItems,
		itemsPerPage,
		variant = "Channel",
		pageParam = "page",
	}: {
		pageNumber: number;
		totalPages: number;
		currentUrl: URL;
		totalItems: number;
		itemsPerPage: number;
		variant?: string;
		pageParam?: string;
	} = $props();

	let currentRanks = $derived({
		// low is 0-indexed
		low: itemsPerPage * pageNumber - itemsPerPage + 1,
		high: itemsPerPage * pageNumber,
	});

	function withPage(page: number): string {
		const url = new URL(currentUrl);
		url.searchParams.set(pageParam, String(page));
		return url.href;
	}

	let hasPrev = $derived(pageNumber > 1);
	let hasNext = $derived(pageNumber < totalPages);
	// let waiting = $state(false);

	function handleKeydown(event: KeyboardEvent) {
		// // idk if i even want this anymore!
		// waiting = true;
		// if (event.key === "ArrowRight" && hasNext) {
		// 	event.preventDefault();
		//
		// 	let newHref = withPage(pageNumber + 1);
		// 	waiting = false;
		// 	goto(newHref, { noScroll: true });
		// } else if (event.key === "ArrowLeft" && hasPrev) {
		// 	event.preventDefault();
		//
		// 	let newHref = withPage(pageNumber - 1);
		// 	waiting = false;
		// 	goto(newHref, { noScroll: true });
		// } else {
		// 	waiting = false;
		// }
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="-mt-6 -mb-3 flex w-full flex-col justify-center">
	<div class="mb-4 flex justify-center">
		<div class="flex w-4/5 flex-row justify-between">
			<PageButton
				icon={ChevronLeft}
				href={withPage(pageNumber - 1)}
				disabled={!hasPrev}
				direction="backward"
				class="mr-0.5"
			/>
			<div class="text-center text-muted-foreground">
				<div class="text-sm">page {pageNumber} of {totalPages}</div>
				<div class="text-xs italic">
					{totalItems} total entries
				</div>
			</div>
			<PageButton
				icon={ChevronRight}
				href={withPage(pageNumber + 1)}
				disabled={!hasNext}
				direction="forward"
				class="ml-0.5"
			/>
		</div>
	</div>
</div>
