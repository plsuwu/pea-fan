<script lang="ts">
	import cn from 'clsx';
	import { Tabs, ScrollArea } from 'bits-ui';
	import { TwitchLogo, User } from 'phosphor-svelte';
	import { fly, slide } from 'svelte/transition';

	import Channels from './Channels.svelte';
	import Chatters from './Chatters.svelte';
	import type { Channel, Chatter } from '$lib/types';
	import { onMount } from 'svelte';
	import { expoIn, expoInOut, expoOut } from 'svelte/easing';

	interface Props {
		data: {
			channels: Channel[];
			chatters: Chatter[];
			leaderboard: { channel: string; leaderboard: any[] } | null;
		};

		onContinueLoad: (key: 'chatter' | 'channel') => Promise<void>;
		hasMoreContent: {
			channels: boolean;
			chatters: boolean;
		};
		loading: boolean;
	}

	let {
		data = $bindable(),
		onContinueLoad,
		hasMoreContent,
		loading = false
	}: Props = $props();

	let channels = $derived(data.channels);
	let chatters = $derived(data.chatters);
	let currentChannel = $derived(data.leaderboard?.channel ?? null);
	let selectedTab: 'channels' | 'chatters' | 'HYDRATING' =
		$state('HYDRATING');

	let expand = $state({ expanded: false, expandable: false });
	onMount(() => {
		if (data.leaderboard != null) {
			chatters = data.leaderboard.leaderboard;
			selectedTab = 'chatters';
			expand.expandable = true;
		} else {
			selectedTab = 'channels';
		}
	});
</script>

<Tabs.Root bind:value={selectedTab} class="flex h-[90.5%] flex-col">
	<Tabs.List
		class="dark:bg-background grid w-full gap-1 space-x-1 rounded-2xl px-4 py-0.5 text-xs font-semibold leading-[0.01em] transition-[grid-template-columns] duration-300 ease-in-out
		{!expand.expandable || selectedTab === 'channels'
			? 'grid-cols-[2fr_2fr]'
			: 'grid-cols-[1fr_3fr]'}"
	>
		<Tabs.Trigger
			value="channels"
			class="dark:data-[state=active]:bg-muted data-[state=active]:border-border shadow-inset border-muted flex h-7
            flex-row items-center justify-around rounded-xl border bg-transparent px-6 py-2 transition-[width] duration-300
            ease-in ease-in-out data-[state=active]:bg-white
            {!expand.expandable ||
			(expand.expandable && selectedTab === 'channels')
				? 'w-full'
				: 'w-[65px]'}
            "
		>
			<TwitchLogo size={16} weight="bold" class="shrink-0" />
			<div
				class="w-min shrink transition-[display] duration-100 ease-in-out
                    {!expand.expandable || selectedTab === 'channels'
					? 'block opacity-100 transition-[opacity] delay-100'
					: 'absolute opacity-0'}
                    "
			>
				channels
			</div>
		</Tabs.Trigger>

		<Tabs.Trigger
			value="chatters"
			class="dark:data-[state=active]:bg-muted data-[state=active]:border-border shadow-inset border-muted flex h-7 flex-row items-center justify-around rounded-xl border bg-transparent px-6 py-2 transition-transform duration-200 data-[state=active]:bg-white"
		>
			<div>chatters</div>
			<User size={16} weight="bold" />
		</Tabs.Trigger>
	</Tabs.List>

	<ScrollArea.Root
		class="border-dark-10 bg-background-alt shadow-card relative my-4 flex max-h-[calc(100vh-100px)]
          min-h-full w-full flex-col overflow-hidden rounded-[10px] border border-r-0 px-1"
	>
		<ScrollArea.Viewport>
			{#key selectedTab}
				<div in:slide={{ duration: 700, easing: expoOut }}>
					<Tabs.Content value="channels" class="shrink pt-3"
						><Channels
							{channels}
							{onContinueLoad}
							hasMoreContent={hasMoreContent.channels}
							{loading}
						/></Tabs.Content
					>

					<Tabs.Content value="chatters" class="shrink-0 pt-3"
						><Chatters
							{chatters}
							{onContinueLoad}
							hasMoreContent={hasMoreContent.chatters}
							{loading}
							forChannel={currentChannel}
						/></Tabs.Content
					>
				</div>
			{/key}
		</ScrollArea.Viewport>
		<ScrollArea.Scrollbar
			orientation="vertical"
			class="bg-muted hover:bg-dark-10 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0
            data-[state=visible]:fade-in-0 shadow-mini-inset flex w-2.5 touch-none select-none rounded-full border-l border-l-transparent
            p-px transition-all duration-200 hover:w-3"
		>
			<ScrollArea.Thumb class="bg-muted-foreground flex-1 rounded-full" />
		</ScrollArea.Scrollbar>
	</ScrollArea.Root>
</Tabs.Root>
