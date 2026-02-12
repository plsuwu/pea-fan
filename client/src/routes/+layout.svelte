<script lang="ts">
	import "./layout.css";
	import "$lib/assets/iosevka.css";
	import favicon from "$lib/assets/favicon.svg";
	import { page } from "$app/state";
	import { onMount } from "svelte";

	import { ModeWatcher } from "mode-watcher";
	import { type ChannelEntry } from "$lib/types/index";
	import Menubar from "$lib/components/menu/menubar.svelte";
	import Noise from "$lib/components/bg/noise.svelte";
	import Channel from "$lib/components/channel/channel.svelte";

	let { data, children } = $props();
	let { channel } = $derived(data);
	let pageTitle = $derived.by(() => {
		let title: string = "piss";
		if (channel) {
			title = `${channel} | ${title}`;
		} else {
			const currPath = page.url.pathname
				.trim()
				.split("/")
				.filter((p) => p !== "");

			const currParams = page.url.searchParams.get("page");

			if (currPath[1]) {
				title = `${currPath[1]}s ${currParams ? `(page ${currParams})` : ""} | ${title}`;
			}
		}

		return title;
	});

	let shortcutHandler: HTMLDivElement;
	onMount(() => {
		if (shortcutHandler) {
			console.log("focusing shortcut handler div");
			shortcutHandler.focus();
		}
	});

	function handleShortcut(event: KeyboardEvent) {
		if (event.ctrlKey) {
			if (event.key === "!") {
				event.preventDefault();
				console.log("!!!!!!");
			} else if (event.key === "@") {
				event.preventDefault();
				console.log("@@@@@@");
			}
		}
	}
</script>

<ModeWatcher />

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{pageTitle}</title>
</svelte:head>

<div
	class="flex h-full w-full flex-col font-iosevka focus:outline-0"
	bind:this={shortcutHandler}
	onkeydown={handleShortcut}
	role="button"
	tabindex="0"
>
	<!-- <Noise> -->
	<div class="flex px-8 py-4">
		<Menubar />
	</div>
	{#if channel}
		<Channel
			{channel}
			leaderboard={data.leaderboard as unknown as ChannelEntry}
		/>
	{/if}
	<div class="flex h-full w-full flex-1 flex-col">
		{@render children()}
	</div>
	<div class="mt-24 flex min-h-[250px] shrink items-end justify-center">
		<div class=" text-center">placeholder</div>
	</div>
	<!-- </Noise> -->
</div>
