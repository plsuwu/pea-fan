<script lang="ts">
	import { Crown, UserCircleDashed, CircleNotch } from 'phosphor-svelte';
	import { formatNumber, PROTO, PF_BASE_URL } from '$lib/utils';
	import { fade } from 'svelte/transition';
	import type { Chatter } from '$lib/types';
	import { onMount } from 'svelte';

	interface Props {
		chatters: Chatter[];

		onContinueLoad: (key: 'user' | 'channel') => void;
		hasMoreContent: boolean;
		loading: boolean;
        forChannel: string | null,
	}

	let {
		chatters = $bindable(),
		onContinueLoad,
		hasMoreContent,
        forChannel,
		loading = false,
	}: Props = $props();

	let scrollContainer: HTMLOListElement | null = $state(null);
	let sentinel: HTMLDivElement | null = $state(null);
	let observer: IntersectionObserver | null = $state(null);

	onMount(() => {
		observer = new IntersectionObserver(
			(entries) => {
				const entry = entries[0];
				if (entry.isIntersecting && hasMoreContent && !loading) {
					onContinueLoad('user');
				}
			},
			{
				root: scrollContainer,
				rootMargin: '100px',
				threshold: 0.7
			}
		);

		if (sentinel) {
			observer.observe(sentinel);
		}

		return () => {
			if (observer) {
				observer.disconnect();
			}
		};
	});

	$effect(() => {
		if (observer && sentinel) {
			observer.observe(sentinel);
		}
	});
</script>

<ol class="flex h-full list-inside list-decimal flex-col px-4 py-4 text-sm">
	{#each chatters as chatter, i}
		<a
			href={`/${chatter.login}`}
			data-sveltekit-preload-data="false"
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
	{/each}
</ol>

{#if loading}
	<div
		in:fade={{ duration: 150 }}
		class="my-2 mb-4 flex w-full animate-spin flex-row items-center justify-center opacity-35"
	>
		<CircleNotch size={28} weight="bold" />
	</div>
{:else if !hasMoreContent}
	<div
		in:fade={{ duration: 150 }}
		class="my-4 mb-4 flex w-full flex-row items-center justify-center text-xs opacity-35"
	>
		no more chatters for this leaderboard :/
	</div>
{/if}

{#if hasMoreContent && !loading}
	<div bind:this={sentinel} class="my-2 h-1 max-h-1 w-full"></div>
{:else}
	<div class="my-2 h-1 max-h-1 w-full"></div>
{/if}
