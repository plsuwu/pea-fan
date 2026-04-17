<script lang="ts">
	import Tenant from "$lib/components/tenant/tenant.svelte";
	import { goto } from "$app/navigation";

	let { data } = $props();
	const { channel, channelData, paginationData } = $derived(data);

	$effect(() => {
		if (!channel) {
			goto("/leaderboard/channel");
		}
	});

	const META_IF_BASE =
		"Leaderboard that ranks Twitch streamers and their chatters based on number of chat messages containing the word piss.";
	const META_IF_CHANNEL = $derived.by(() =>
		channelData
			? `Leaderboard for ${channelData.name}'s top Twitch chatters ranked by the number of chat messages about piss.`
			: ""
	);

	let content = $derived.by(() => {
		if (channel) {
			return META_IF_CHANNEL;
		}

		return META_IF_BASE;
	});
</script>

<svelte:head>
	<meta name="description" {content} />
</svelte:head>

{#if channel && channelData && paginationData}
	<Tenant channelLogin={channel} {channelData} {paginationData} />
{/if}
