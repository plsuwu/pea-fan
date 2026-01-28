<script lang="ts">
	import type { ChannelEntry, Entry } from "$lib/types/index.js";
	import { getAltImageSizeUrl } from "$lib/utils/index.js";

	let { data } = $props();
	let { channel } = $derived(data);
	let leaderboard = $derived(data.leaderboard as unknown as ChannelEntry);
</script>

<div class="flex flex-col">
	{#if channel && leaderboard}
		<div class="flex space-x-4 px-10 py-16">
			<div>{"*)"}</div>
			<div>{channel}</div>
			<div>{"==>"}</div>
			<div>{leaderboard.total_channel}</div>
		</div>
		<ol>
			{#each leaderboard.chatter_scores as score}
				<li class="flex items-center space-x-4">
					<img
						src={getAltImageSizeUrl(score.chatter_image, "XS")}
						alt={score.chatter_login}
						class="rounded-full"
					/>
					<div>{score.chatter_name}</div>
					<div>{score.score}</div>
				</li>
			{/each}
		</ol>
	{/if}
</div>
