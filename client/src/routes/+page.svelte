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
		"Leaderboard ranking a handful of Twitch broadcasters (primarily vtubers) by the number of times a chatter has sent a message about piss to their chat room.";
	const META_IF_CHANNEL = $derived(
		`Leaderboard of chatters in Twitch broadcaster ${channelData!.name}'s chat room, whereby chatters are ranked based on the number of their chat messages mentioning piss.`
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
