<script lang="ts">
	import "./layout.css";
	import "$lib/assets/iosevka.css";
	import favicon from "$lib/assets/favicon.svg";
	import Menubar from "$lib/components/menu/menubar.svelte";
	import { ModeWatcher } from "mode-watcher";
	import { ChannelEntry } from "$lib/types/index.js";
	import Channel from "$lib/components/channel/channel.svelte";

	let { data, children } = $props();
	let { channel } = $derived(data);
</script>

<ModeWatcher />

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>"a"</title>
</svelte:head>

<div class="flex h-full w-full flex-col font-iosevka">
	<div class="flex justify-end px-8 py-4">
		<Menubar />
	</div>
	{#if channel}
		<Channel
			{channel}
			leaderboard={data.leaderboard as unknown as ChannelEntry}
		/>
	{/if}
	{@render children()}
</div>
