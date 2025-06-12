<script lang="ts">
	import type { ChannelResponse } from '$lib/client/constants';
	import ChannelRoute from '$lib/components/ChannelRoute.svelte';
	import { ROOT_HOSTNAME } from '$lib/client/constants';

	let {
		data
	}: {
		data: {
			count: number;
			channels: { channel: string; live: boolean; color: string; total: string }[];
			channelData: ChannelResponse;
			color: string;
		};
	} = $props();

	let channelData = data.channelData;
	let channels = data.channels;
	let channel = data.channels[0].channel;
	let color = data.color;
	let total = data.channelData?.total ?? null;
	let leaderboard = data.channelData?.leaderboard ?? null;

	function sortByLiveState(
		channels: { channel: string; live: boolean; color: string; total: string }[]
	) {
		return channels.sort((a, b) => {
			return Number(a.total) < Number(b.total) ? 1 : -1;
		});
	}

	function getChannelUri(channel: string) {
		return `http://${channel}.${ROOT_HOSTNAME}`;
	}
</script>

{#if data.count === 1 && total && channelData}
	<div>
		<ChannelRoute {channel} {total} {color} {leaderboard} />
	</div>
{:else}
	<div class="w-full sm:w-3/4 xl:w-2/4 2xl:w-2/6 pt-8">
		{#each sortByLiveState(channels) as channel, i}
			<a
				href={`${getChannelUri(channel.channel)}`}
				style={`background-color: ${i % 2 === 0 ? '#4a556550' : ''}`}
				class="transition-opacity duration-150 hover:opacity-50 flex flex-row px-2 py-1"
			>
            <div class="pr-2">{i + 1}.</div>
				<div 
                class="flex w-full flex-row justify-between " >
					<div>{channel.channel}</div>
					<div>({channel.total})</div>
				</div>
			</a>
		{/each}
	</div>
{/if}
