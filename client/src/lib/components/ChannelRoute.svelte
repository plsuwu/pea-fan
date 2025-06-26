<script lang="ts">
	import { ROOT_HOSTNAME } from '$lib/client/constants';
	import Crown from '$lib/components/icons/Crown.svelte';

	let { color, total, channel, leaderboard } = $props();

    function lPad(n: number) {
        let text = (n).toString(); 

        if (text.length < 2) {
            return '&nbsp;' + text
        } 

        return text;
    }
</script>

<div class="flex h-full flex-col items-center pt-8 overflow-x-hidden w-full">
	<div class="w-5/6 py-16 text-justify lg:w-full">
		<span class="">piss</span> has been brought up a total of
		<span class="font-bold underline">{total}</span>
		{total == 1 ? 'time' : 'times'} in
		<span
			class="font-semibold italic transition-opacity duration-150 hover:opacity-50"
			style={`color: ${color};`}
			><a href={`https://twitch.tv/${channel}`} target="_blank">{channel}'s</a></span
		> chat.
	</div>

	{#if leaderboard.length > 0}
		<div class="mb-4">these chatters have brought piss up the most:</div>
		<div class="h-[550px] w-full overflow-y-scroll p-4 border rounded-md">
			{#each leaderboard as chatter, i}
				<div class="flex flex-row justify-between px-4">
					<div class="flex flex-row items-center justify-center py-px">
						{#if i === 0}
							<Crown size={22} class="mr-4" />
						{:else}
							<Crown size={22} class="invisible ml-4" />
						{/if}
						<div class="font-mono">
							{@html lPad(i + 1)}.
						</div>
						<div class="px-4">{chatter[0]}</div>
					</div>
					<div class="px-4 pt-2 text-sm">
						(<span class="font-semibold"
							>{chatter[1]}</span> <span>{chatter[1] === 1 ? 'time' : 'times'}</span
						>)
					</div>
				</div>
			{/each}
		</div>
	{/if}
	<!-- <a href={`https://${ROOT_HOSTNAME}`} class="hover:opacity-50 transition-opacity duration-150 pb-12">{'<<<'} back</a> -->
	<a
		href={`https://${ROOT_HOSTNAME}`}
		class="mb-12 mt-12 rounded-xl border px-2 py-1 transition-opacity duration-150 hover:opacity-50"
		>{'<<<'} back</a
	>
</div>
