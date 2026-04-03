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

{#if channel && channelData && paginationData}
	<Tenant channelLogin={channel} {channelData} {paginationData} />
{/if}
