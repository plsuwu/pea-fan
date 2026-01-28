<script lang="ts">
	import { URLS } from "$lib";
	import type { ChatterScore, ChannelScore } from "$lib/types";
	import { getAltImageSizeUrl } from "$lib/utils";
	let {
		channel,
		entries
	}: { channel: string; entries: (ChatterScore | ChannelScore)[] } = $props();
	type Score = {
		id: string;
		login: string;
		name: string;
		color: string;
		image: string;
	};

	let typedEntries: Score[] = $derived(
		getScoreFields((entries ?? []).slice(0, 7))
	);
	let remaining = $derived.by(() => {
		if (!entries) return 0;

		return Math.max(0, entries.length - typedEntries.length);
	});

	function getScoreFields(entry: (ChatterScore | ChannelScore)[]) {
		if (entry.length === 0) return new Array();

		if ((entry[0] as ChatterScore).chatter_name) {
			const entries = entry as ChatterScore[];
			return entries.map((e) => ({
				id: e.chatter_id,
				login: e.chatter_login,
				name: e.chatter_name,
				color: e.chatter_color,
				image: e.chatter_image
			}));
		} else {
			const entries = entry as ChannelScore[];
			return entries.map((e) => ({
				id: e.channel_id,
				login: e.channel_login,
				name: e.channel_name,
				color: e.channel_color,
				image: e.channel_image
			}));
		}
	}
</script>

{#snippet Image(entry: Score, idx: number)}
	{@const src = getAltImageSizeUrl(entry.image, "SM")}
	<img
		{src}
		alt={entry.login}
		class="size-8 rounded-full bg-background p-px outline-1
        outline-foreground"
		style={`z-index: ${10 - idx}`}
	/>
{/snippet}

<div class="group flex items-center justify-end px-8">
	{#if typedEntries.length > 0}
		<div class="flex -space-x-2">
			{#each typedEntries as entry, idx}
				{@render Image(entry, idx)}
			{/each}
		</div>
		{#if remaining > 0}
			<a href={`${URLS().proto}://${channel}.${URLS().base}`} class="">
				<div
					class="ml-1 transition-discrete duration-200 ease-out
                    opacity-50 hover:opacity-100 lg:block hidden"
				>
					... <span class="text-xs">({remaining} more)</span>
				</div>
			</a>
		{/if}
	{/if}
</div>
