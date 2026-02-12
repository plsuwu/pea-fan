<script lang="ts">
	import type { Entry } from "$lib/types";
	import * as Tt from "$lib/shadcn-components/ui/tooltip";
	import Square from "../misc/square.svelte";
	import { mode } from "mode-watcher";
	import { getAltImageSizeUrl, intoUntypedEntry, readableColor } from "$lib/utils";
	import { cn } from "$lib/shadcn-components/utils";
	import Hover from "./hover.svelte";

	let {
		unknownEntry,
		index = 0
	}: {
		unknownEntry: Entry;
		index?: number;
	} = $props();

	let imgLoaded = $state(false);
	let entry = $derived(intoUntypedEntry(unknownEntry));

	const containerClass = $derived(
		mode.current === "light" ? "shadow-container" : "shadow-container-dark"
	);

</script>

<div
	class={cn(
		`group relative flex items-center justify-between space-x-4
            rounded-[3px] border-2 border-foreground bg-background
            p-4 transition-all ease-in hover:-translate-y-2`,
		containerClass
	)}
>
	<div class="flex flex-row space-x-6">
		<div>
			<div class="group flex items-center justify-center p-4">
				<div class="absolute text-xl font-bold">{entry.ranking}</div>
				<div class="absolute bg-transparent transition-all duration-100">
					<Square size={40} />
				</div>
			</div>
		</div>
		<div class="flex items-center">
			<img
				src={getAltImageSizeUrl(entry.image, "SM")}
				alt={entry.login}
				class="bg-muted p-1"
			/>
		</div>
		<div class="flex flex-col">
			<span
				class="text-xl font-medium"
				style={`color: ${readableColor(entry.color)};`}>{entry.name}</span
			>
			{#if entry.totalScores > 0}
				<div
					class="mt-3 ml-3 text-sm -tracking-wider text-accent-foreground/55 italic"
				>
					mentioned most by:
				</div>
				<div class="mt-2 ml-3 flex flex-row space-x-2">
					<Tt.Provider>
						<div class="flex flex-row -space-x-1">
							{#each entry.scores?.slice(0, 6) as subentry}
								<div>
									<Hover
										login={subentry.login}
										name={subentry.name}
										color={subentry.color}
										imgUrl={subentry.image}
										total={subentry.score}
									></Hover>
								</div>
							{/each}
						</div>
					</Tt.Provider>
					<div class="flex flex-row items-end">
						{#if entry.totalScores > 6}
							<span
								class="ml-1 text-sm font-semibold -tracking-wider
                                text-accent-foreground/55 italic"
							>
								+ {entry.totalScores} others
							</span>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>
	<div class="shrink-0 text-right">
		<div class="flex flex-col">
			<span class="text-2xl font-bold">
				{entry.total.toLocaleString()}
			</span>
			<span>mentions</span>
		</div>
	</div>
</div>
