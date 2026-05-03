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
		"Piss.fan is a leaderboard ranking a handful of Twitch VTubers based on the number of piss-related messages sent to their chat rooms.";
	const META_IF_CHANNEL = $derived(
		`Piss.fan leaderboard for ${channelData!.name}'s chat room, ranking chatters by the number of times they have sent a message about piss`
	);

	let content = $derived.by(() => {
		return channel ? META_IF_CHANNEL : META_IF_BASE;
	});
</script>

<svelte:head>
	<meta name="description" {content} />
</svelte:head>

{#if channel && channelData && paginationData}
	<Tenant channelLogin={channel} {channelData} {paginationData} />
{/if}
