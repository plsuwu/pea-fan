<script lang="ts">
	import { ROOT_HOSTNAME } from '$lib/client/constants';

	let { color, total, channel, leaderboard } = $props();
</script>

<div class="flex h-full w-full flex-col items-center pt-8">
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
		<div>these chatters have brought piss up the most:</div>
		<div class="w-full py-8">
			{#each leaderboard as chatter}
				<div class="flex flex-row justify-between px-4">
					<div class="px-4 pt-2">{chatter[0]}</div>
					<div class="px-4 pt-2 text-sm">
						(in <span class="font-semibold italic"
							>{chatter[1]} {chatter[1] === 1 ? 'message' : 'messages'}</span
						>)
					</div>
				</div>
			{/each}
		</div>
	{/if}
	<a href={`http://${ROOT_HOSTNAME}`} class="hover:opacity-50 transition-opacity duration-150">{'<<<'} back</a>
</div>
