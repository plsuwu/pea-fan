<script lang="ts">
	import { Crown, UserCircleDashed } from 'phosphor-svelte';
	import { formatNumber, PROTO, PF_BASE_URL } from '$lib/utils';
	import type { Chatter } from '$lib/types';

	let { chatters }: { chatters: Chatter[] } = $props();
</script>

<ol class="flex h-full list-inside list-decimal flex-col px-4 py-4 text-sm">
	{#each chatters as chatter, i}
		<a
			href={`/${chatter.login}`}
			class="transition-opacity duration-100 ease-out hover:opacity-50"
		>
			<li class="flex flex-row items-center justify-start space-y-1">
				{#if i === 0}
					<Crown size={16} class="shrink-0 text-yellow-400" />
				{:else}
					<Crown size={16} class="invisible shrink-0" />
				{/if}
				<div class="pr-1 font-mono">{@html formatNumber(i)}.</div>
				<div class="shrink-0 pl-1.5 pr-3">
					{#if chatter.image}
						<img
							src={chatter.image}
							class="size-5 rounded-full"
							alt={`${chatter.login} profile_image`}
						/>
					{:else}
						<UserCircleDashed class="size-5" />
					{/if}
				</div>
				<div class="flex w-full flex-row items-center justify-between">
					<div>{chatter.login}</div>
					<div>{chatter.total}</div>
				</div>
			</li>
		</a>

		<!-- <li class="flex flex-row items-center justify-start space-x-2"> -->
		<!-- 	{#if i === 0} -->
		<!-- 		<Crown size={16} class="text-yellow-400" /> -->
		<!-- 	{:else} -->
		<!-- 		<Crown size={16} class="invisible" /> -->
		<!-- 	{/if} -->
		<!-- 	<div class="font-mono">{@html formatNumber(i)}.</div> -->
		<!-- 	<div>{chatter.login}</div> -->
		<!-- </li> -->
	{/each}
</ol>
