<script lang="ts">
	import type { PageServerData } from './$types';

	let { data }: { data: PageServerData } = $props();
	let { userLeaderboard } = data;
</script>

<ul class="w-full">
	{#each userLeaderboard as channel}
		<li class="flex w-full flex-row justify-around">
			<div>{channel.name}</div>
			<div>{channel.score}</div>
		</li>
	{/each}
</ul>
