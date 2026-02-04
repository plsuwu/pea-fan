<script lang="ts">
	import { capitalize, mapPagedResponseToEntries } from "$lib/utils";
	import type { PageData } from "./$types";
	import Table from "$lib/components/leaderboard/table.svelte";
	import Pagination from "$lib/components/leaderboard/filtering/pagination.svelte";

	let { data }: { data: PageData } = $props();
	let leaderboard = $derived(data.leaderboardData);
	let variant = $derived(
		capitalize<"Chatter" | "Channel">(data.leaderboardVariant)
	);

	let entries = $derived(mapPagedResponseToEntries(leaderboard, variant));
</script>

<div class="mx-auto mt-8 pb-14 text-4xl font-bold">
	{variant} Leaderboard
</div>
<div class="w-11/12 self-center lg:w-4/5 2xl:w-1/2">
	<div class="rounded-md border">
		<Table {entries} />
	</div>
	<div class="mt-8 flex w-full items-center justify-between">
		<Pagination />
	</div>
</div>
