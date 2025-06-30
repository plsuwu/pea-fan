<script lang="ts">
	import { Tabs, ScrollArea } from 'bits-ui';
	import { TwitchLogo, User } from 'phosphor-svelte';

	import Channels from './Channels.svelte';
	import Chatters from './Chatters.svelte';
	import type { Channel, Chatter } from '$lib/types';

	let { data }: { data: { channels: Channel[]; chatters: Chatter[] } } =
		$props();

	let channels = data.channels;
	let chatters = data.chatters;
</script>

<Tabs.Root value="channels" class="flex h-[90.5%] flex-col">
	<Tabs.List
		class="dark:bg-background grid w-full grid-cols-2 gap-1 space-x-1 rounded-2xl px-4 py-0.5 text-xs font-semibold leading-[0.01em]"
	>
		<Tabs.Trigger
			value="channels"
			class="dark:data-[state=active]:bg-muted data-[state=active]:border-border shadow-inset border-muted flex h-7 flex-row 
            items-center justify-around rounded-xl border bg-transparent px-6 py-2 data-[state=active]:bg-white"
		>
			<TwitchLogo size={16} weight="bold" />
			<div>channels</div>
		</Tabs.Trigger>

		<Tabs.Trigger
			value="users"
			class="dark:data-[state=active]:bg-muted data-[state=active]:border-border shadow-inset border-muted flex h-7 flex-row
            items-center justify-around rounded-xl border bg-transparent px-6 py-2 data-[state=active]:bg-white"
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
			<Tabs.Content value="channels" class="pt-3"
				><Channels {channels} /></Tabs.Content
			>
			<Tabs.Content value="users" class="pt-3"
				><Chatters {chatters} /></Tabs.Content
			>
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
