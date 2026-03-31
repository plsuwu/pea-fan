<script lang="ts">
	import "./layout.css";
	import "$lib/assets/iosevka.css";
	import favicon from "$lib/assets/favicon.svg";

	import type { LayoutData } from "./$types";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { navigating, page } from "$app/state";

	import { mode, ModeWatcher, setMode } from "mode-watcher";
	import * as Tt from "$lib/shadcn-components/ui/tooltip/index";
	import { MoveLeft, X } from "@lucide/svelte";

	// import Noise from "$lib/components/bg/noise.svelte";
	import {
		setModeCookie,
		getModeCookie,
		getParentDomain,
	} from "$lib/utils/mode-cookie.svelte";
	import { Rh } from "$lib/utils/route";

	import Footer from "$lib/components/menu/footer.svelte";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import Stats from "$lib/components/tenant/stats/stats.svelte";
	import ExternalLinks from "$lib/components/tenant/stats/external.svelte";
	import Menubar from "$lib/components/menu/menubar.svelte";
	import { slide } from "svelte/transition";
	import { Button } from "$lib/shadcn-components/ui/button";
	import { expoInOut, expoOut } from "svelte/easing";

	let { data, children } = $props();
	let { hostname } = page.url;

	let {
		channel,
        liveBroadcasters,
		scoreWindows,
		channelData,
		announcement,
		announcementClearToken,
	}: LayoutData = $derived(data);

	let waitForLoad = $derived(navigating.to !== null);
	let announcementCleared = $state(false);

	onMount(() => {
		const saved = getModeCookie();
		if (saved === "dark" || saved === "light" || saved === "system") {
			setMode(saved);
		}

		if (announcementClearToken) {
			announcementCleared = true;
		}
	});

	$effect(() => {
		const current = mode.current;
		if (current) {
			setModeCookie(current);
		}
	});

	function handleClearAnnouncement() {
		announcementCleared = true;
		document.cookie = `seen-announcement=${announcement?.hash}; domain=${getParentDomain()}; path=/; max-age=forever;`;
	}

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
			channel
				? (window.location.href = location)
				: goto(location, { noScroll: false });
		}
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{pageTitle}</title>
</svelte:head>

<svelte:window onkeydown={handleShortcut} />
<ModeWatcher />

<div class="flex h-[110vh] w-full flex-col font-iosevka">
	<header
		class="mt-4 mb-2 flex flex-row items-center justify-between px-8 py-4"
	>
		<Menubar />
	</header>

	<div class="border-t-2"></div>
	{#if announcement && !announcementCleared && !announcementClearToken}
		<div
			transition:slide={{ duration: 250, axis: "y", easing: expoInOut }}
			class="flex max-h-[105px] w-full flex-row items-center justify-between bg-amber-200
            px-2 py-1 text-xs text-black lg:px-8 lg:py-2 lg:text-base"
		>
			<div></div>
			<div class="text-[11px] font-semibold sm:text-sm lg:text-base">
				{@html announcement.content}
			</div>
			<Button
				onclick={handleClearAnnouncement}
				variant="ghost"
				size="icon-sm"
				class="ml-2 flex flex-row items-center justify-center rounded-full border
                text-black hover:cursor-pointer hover:text-black"
			>
				<X size={3} />
			</Button>
		</div>
	{/if}

	<main class="mb-32 flex-1">
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
			<!-- <div -->
			<!-- 	class="fixed top-0 left-0 z-10 flex h-full w-full flex-row items-center -->
			<!--                 justify-center bg-background/50 backdrop-blur-[2px]" -->
			<!-- > -->
			<!-- 	<Spinner class="size-5" /> -->
			<!-- </div> -->
			<div class="mt-12 flex h-full min-h-[450px] flex-col">
				<div class="w-[93%] self-center xl:w-[90%]">
					<div class="flex flex-col lg:flex-row xl:justify-center">
						{#if channel && channelData && scoreWindows}
							<div
								class="w-full px-2 md:min-w-[320px] lg:pr-6 lg:pl-2 xl:w-[35%]"
							>
								<Stats channelLogin={channel} {channelData} {scoreWindows}>
									<ExternalLinks
										channelLogin={channelData.login}
										channelId={channelData.id}
									/>
								</Stats>
							</div>
						{/if}
						{@render children()}
					</div>
				</div>
			</div>
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
