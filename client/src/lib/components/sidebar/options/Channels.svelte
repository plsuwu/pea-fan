<script lang="ts">
	import { Crown } from 'phosphor-svelte';
	import { formatNumber, PROTO, PF_BASE_URL } from '$lib/utils';
	import type { Channel } from '$lib/types';

	let { channels }: { channels: Channel[] } = $props();
</script>

<ol class="list-inside list-decimal flex-col px-4 py-4 text-sm">
	{#each channels as chan, i}
		<a 
        href={`${PROTO}://${chan.broadcaster}.${PF_BASE_URL}/`}
        class="hover:opacity-50 transition-opacity duration-100 ease-out"
        >
			<li class="flex flex-row items-center justify-start space-y-1">
				{#if i === 0}
					<Crown size={16} class="shrink-0 text-yellow-400" />
				{:else}
					<Crown size={16} class="invisible shrink-0" />
				{/if}
				<div class="pr-1 font-mono">{@html formatNumber(i)}.</div>
				<div class="shrink-0 pl-1.5 pr-3">
					<img
						src={chan.image}
						class="size-5 rounded-full"
						alt={`${chan.broadcaster} profile_image`}
					/>
				</div>
				<div class="flex w-full flex-row items-center justify-between">
					<div>{chan.broadcaster}</div>
					<div>{chan.total}</div>
				</div>
			</li>
		</a>
	{/each}
</ol>
