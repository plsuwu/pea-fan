import {
	renderComponent,
	renderSnippet
} from "$lib/shadcn-components/ui/data-table";
import type { ChannelEntry, ChatterEntry, Entry } from "$lib/types";
import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";
import Nameplate from "./nameplate.svelte";
import SubLeaderboard from "./subleaderboard.svelte";

export const columns: ColumnDef<Entry>[] = [
	{
		accessorKey: "ranking",
		header: () => {
			const snippet = createRawSnippet(() => ({
				render: () =>
					`<div class="text-end w-2/3">
                    Rank
                </div>`
			}));

			return renderSnippet(snippet);
		},
		cell: ({ row }) => {
			const r = `#${row.original.data.ranking}`;
			const snippet = createRawSnippet(() => ({
				render: () =>
					`<div class="text-end text-lg font-semibold italic w-2/3">
                    ${r}
                </div>`
			}));

			return renderSnippet(snippet);
		}
	},
	{
		accessorKey: "data.name",
		header: ({ table }) => {
			return table.getCoreRowModel().rows.at(0)?.original._tag;
		},
		cell: ({ row }) => {
			return renderComponent(Nameplate, { entry: row.original.data });
		}
	},
	{
		header: "Score",
		accessorFn: (row) => {
			if (row._tag === "Channel") {
				return (row.data as ChannelEntry).total_channel;
			} else {
				return (row.data as ChatterEntry).total;
			}
		}
	},
	{
		accessorKey: "scoreboard",
		header: ({ table }) => {
			const dataType = table.getCoreRowModel().rows.at(0)?.original._tag;
			const content = `Top ${dataType === "Channel" ? "Chatter" : "Channel"}s`;

			const snippet = createRawSnippet(() => ({
				render: () => `<div class="text-end px-8">${content}</div>`
			}));

			return renderSnippet(snippet);
		},
		cell: ({ row }) => {
			const data =
				row.original._tag === "Channel"
					? (row.original.data as ChannelEntry).chatter_scores
					: (row.original.data as ChatterEntry).channel_scores;

			return renderComponent(SubLeaderboard, {
				channel: row.original.data.login,
				entries: data
			});
		}
	}
];
// accessorFn: (row) => {
// 	if (row._tag === "Channel") {
// 		return (row.data as ChannelEntry).chatter_scores;
// 	} else {
// 		return (row.data as ChatterEntry).channel_scores;
// 	}
// },
