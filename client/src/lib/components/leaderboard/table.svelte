<script lang="ts">
	import { fade } from "svelte/transition";
	import { columns } from "./columns";
	import { type ColumnDef, getCoreRowModel } from "@tanstack/table-core";
	import {
		createSvelteTable,
		FlexRender
	} from "$lib/shadcn-components/ui/data-table";
	import * as Table from "$lib/shadcn-components/ui/table";
	import type { Entry } from "$lib/types";
	import Row from "./row.svelte";
	import { page } from "$app/state";
	import { URLS } from "$lib";

	let { entries }: { entries: Entry[] } = $props();
	let param = $derived(page.url.searchParams.get("page") || "0");

	// const table = createSvelteTable({
	// 	get data() {
	// 		return entries;
	// 	},
	// 	// svelte-ignore state_referenced_locally
	// 	columns,
	// 	getCoreRowModel: getCoreRowModel()
	// });
</script>

<div class="flex flex-col space-y-2">
	{#each entries as entry, idx}
		<a href={`${URLS().proto}://${entry.data.login}.${URLS().base}`}>
			<Row unknownEntry={entry} index={idx} />
		</a>
	{/each}
</div>

<!-- <Table.Root> -->
<!-- 	<Table.Header> -->
<!-- 		{#each table.getHeaderGroups() as headerGroup (headerGroup.id)} -->
<!-- 			<Table.Row> -->
<!-- 				{#each headerGroup.headers as header (header.id)} -->
<!-- 					<Table.Head colspan={header.colSpan}> -->
<!-- 						{#if !header.isPlaceholder} -->
<!-- 							<FlexRender -->
<!-- 								content={header.column.columnDef.header} -->
<!-- 								context={header.getContext()} -->
<!-- 							/> -->
<!-- 						{/if} -->
<!-- 					</Table.Head> -->
<!-- 				{/each} -->
<!-- 			</Table.Row> -->
<!-- 		{/each} -->
<!-- 	</Table.Header> -->
<!-- 	<Table.Body> -->
<!-- 		{#each table.getRowModel().rows as row (row.id)} -->
<!-- 			<Table.Row data-state={row.getIsSelected() && "selected"}> -->
<!-- 				{#each row.getVisibleCells() as cell (cell.id)} -->
<!-- 					<Table.Cell> -->
<!-- 						<FlexRender -->
<!-- 							content={cell.column.columnDef.cell} -->
<!-- 							context={cell.getContext()} -->
<!-- 						/> -->
<!-- 					</Table.Cell> -->
<!-- 				{/each} -->
<!-- 			</Table.Row> -->
<!-- 		{/each} -->
<!-- 	</Table.Body> -->
<!-- </Table.Root> -->
