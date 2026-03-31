<script lang="ts">
	import type { PageData } from "./$types";
	import Tenant from "$lib/components/tenant/tenant.svelte";
	import { intoParentEntry, type UntypedEntry } from "$lib/utils/index.js";
	import Stats from "$lib/components/tenant/stats/stats.svelte";
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
