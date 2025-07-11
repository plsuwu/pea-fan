<script lang="ts">
	import type { PageServerData } from './$types';
	import Key from '$lib/components/Key.svelte';
	import LiveCard from '$lib/components/broadcaster-card/LiveCard.svelte';

	let { data } = $props();
	let { leaderboard, channels, chatters, live } = data;

	console.log(live);
</script>

{#if live}
	<LiveCard broadcasters={live} />
{/if}
