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
		"leaderboard ranking a handful of twitch broadcasters by the number of times chatter messages have contained the word piss during a stream";
	const META_IF_CHANNEL = $derived(
		`leaderboard ranking chatters in twitch broadcaster ${channelData!.name}'s chat room based on the number of times their messages have contained the word piss`
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
