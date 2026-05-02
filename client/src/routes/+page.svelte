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


</script>

<svelte:head>
	<meta name="description"
    content="Piss Fan is a leaderboard ranking a handful of Twitch VTubers based on the number of piss-related messages sent to their chat rooms."
    />
</svelte:head>

{#if channel && channelData && paginationData}
	<Tenant channelLogin={channel} {channelData} {paginationData} />
{/if}
