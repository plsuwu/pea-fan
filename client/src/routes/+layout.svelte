<script lang="ts">
	import "./layout.css";
	import "$lib/assets/iosevka.css";
	import favicon from "$lib/assets/favicon.svg";
	import { navigating, page } from "$app/state";
	import * as Tt from "$lib/shadcn-components/ui/tooltip/index";
	import { mode, ModeWatcher, setMode } from "mode-watcher";
	import { setModeCookie, getModeCookie } from "$lib/utils/mode-cookie";
	import { onMount } from "svelte";

	import Menubar from "$lib/components/menu/menubar.svelte";
	// import Noise from "$lib/components/bg/noise.svelte";
	import { goto } from "$app/navigation";
	import { Rh } from "$lib/utils/route";
	import { MoveLeft } from "@lucide/svelte";
	import { fade } from "svelte/transition";
	import { expoIn, expoOut } from "svelte/easing";
	import Footer from "$lib/components/menu/footer.svelte";
	import ModeChanger from "$lib/components/menu/mode-changer.svelte";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";

	let { data, children } = $props();
	let { channel } = $derived(data);
	let { hostname } = page.url;

	let waitForLoad = $derived(navigating.to !== null);
	// let waitForLoad = true;

	// TODO:
	//  move this + the keyboard handler into a utility function (or
	//  perhaps standalone class store)
	let pageTitle = $derived.by(() => {
		let title: string = "piss fan";
		const rawPath = page.url.pathname;

		if (channel) {
			title = `${channel} | ${title}`;
		} else if (rawPath !== "/admin" && rawPath !== "/admin/login") {
			const currPath = page.url.pathname
				.trim()
				.split("/")
				.filter((p) => p !== "");

			const currParams = page.url.searchParams.get("page");
			if (currPath[1]) {
				title = `${currPath[1]}s ${
					currParams ? `| page ${currParams}` : ""
				} | ${title}`;
			}
		}

		return title;
	});

	function getNextUrlBase(path: string) {
		const next = new URL(Rh.deriveBase(hostname));
		next.pathname = path;
		next.searchParams.set("page", "1");

		return next;
	}

	function handleShortcut(event: KeyboardEvent) {
		if (event.ctrlKey) {
			let location;
			switch (event.key) {
				case "C":
					event.preventDefault();
					location = getNextUrlBase("/leaderboard/channel").href;
					break;

				case "V":
					event.preventDefault();
					location = getNextUrlBase("/leaderboard/chatter").href;
					break;

				default:
					break;
			}

			if (!location) return;

			// cannot call `goto()` on this location if we are on a tenant
			channel ? (window.location.href = location) : goto(location);
		}
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{pageTitle}</title>
</svelte:head>

<svelte:window onkeydown={handleShortcut} />
<ModeWatcher />

<div class="flex h-[111vh] w-full flex-col font-iosevka">
	<header
		class="mt-4 mb-2 flex flex-row items-center justify-between px-8 py-4"
	>
		<Menubar />
		<ModeChanger />
	</header>

	<main class="mb-32 flex-1">
		<div class="border-t-2 pt-2"></div>
		{#if channel}
			<div class="hidden w-[90%] justify-self-center lg:block">
				<a
					href={`${Rh.proto}://${Rh.deriveBase(page.url.host)}/leaderboard/channel`}
					class="flex w-max items-center justify-start px-8 pt-2 text-accent-foreground/15
                        transition-all duration-100 ease-in hover:text-accent-foreground/35"
				>
					<MoveLeft strokeWidth={1} size={20} />
					<div class="ml-2 text-[13px]">all channels</div>
				</a>
			</div>
		{/if}
		<Tt.Provider delayDuration={0}>
			{#if waitForLoad}
				<div
					class="fixed top-0 left-0 flex h-full w-full flex-row items-center justify-center
                    bg-background/50 backdrop-blur-[2px] z-10"
				>
					<Spinner class="size-5" />
				</div>
			{:else}
				<div class="mt-12 flex h-full min-h-[450px] flex-col">
					{@render children()}
				</div>
			{/if}
		</Tt.Provider>
	</main>

	<footer class="mb-4 border-t-2">
		<div
			class="mx-8 flex h-[150px] max-h-[150px] min-h-[150px] flex-row items-center"
		>
			<Footer />
		</div>
	</footer>
</div>
