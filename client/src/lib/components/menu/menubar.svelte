<script lang="ts">
	import { type Component } from "svelte";
	import { page } from "$app/state";
	import {
		Tv,
		House,
		MenuIcon,
		SearchIcon,
		UsersRound,
		ChevronDown,
		CircleQuestionMarkIcon,
		RouterIcon,
		NotebookIcon,
		StickyNoteIcon,
		EllipsisVerticalIcon,
		ToggleLeft,
		ToggleRight,
	} from "@lucide/svelte";
	import { Rh } from "$lib/utils/route";
	import * as Dropdown from "$lib/shadcn-components/ui/dropdown-menu";
	import * as BGrp from "$lib/shadcn-components/ui/button-group";
	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import SearchInput from "./search.svelte";
	import { slide } from "svelte/transition";
	import { expoOut } from "svelte/easing";
	import { debounce, type SearchResult } from "./search-handler.svelte";
	import ModeChanger from "./mode-changer.svelte";
	import { mode, toggleMode } from "mode-watcher";
	import { setModeCookie } from "$lib/utils/mode-cookie.svelte";

	const BASE_HOST_URL = `${Rh.proto}://${Rh.deriveBase(page.url.host)}`;
	const BASE_API_URL = `${Rh.apiv1}`;

	const searchDebounceCallback = debounce(handleSearch, 350);

	async function handleSearch(signal: AbortSignal, query: string) {
		const login = query.trim();

		// clean up state and return if empty query
		if (!login || login === "") {
			current = "";
			previous = "";

			return;
		}

		try {
			const queryUrl = () => {
				const url = new URL(`${BASE_API_URL}/search/${login}`);
				return url;
			};

			const res = await fetch(queryUrl().href, { signal });
			if (!signal.aborted) {
				const body = await res.json();
				console.log(body.data);

				results = body.data[0].map((result: any) => {
					return {
						...result,
						page: Math.ceil(result.ranking / 15),
					};
				});
				// for (const result of results) {
				// 	getPageFromRank(result.ranking, body.data[0]);
				// }
			}
		} finally {
			previous = current;
			loading = false;
		}
	}

	let inputRef = $state<HTMLElement | null>(null);
	let loading = $state(false);

	let current = $state("");
	let previous = $state("");

	let results = $state<SearchResult[]>([]);

	let inputVisible = $state(true);

	$effect(() => {
		if (current !== previous && !!current) {
			loading = true;
			searchDebounceCallback(current);
		}
	});

	const ROUTE = {
		channel: "/leaderboard/channel",
		chatter: "/leaderboard/chatter",
		about: "/about",
		bot: "/bot",
	} as const;

	type Route = keyof typeof ROUTE;

	function routeUrl(route: Route) {
		return new URL(BASE_HOST_URL + ROUTE[route]);
	}

	function handleKeydown(event: KeyboardEvent) {
		const { key, ctrlKey } = event;
		if (key === "j" && ctrlKey && inputRef != null) {
			event.preventDefault();
			inputRef.focus();
		}
	}

	function toggleModeCookie() {
		toggleMode();
		setModeCookie(mode.current!);
	}
</script>

{#snippet NavButton(
	href: string,
	Icon: Component,
	title?: string,
	childProps?: Record<string, unknown>,
	iconProps?: Record<string, unknown>,
	ariaLabel: string = "navigation button"
)}
	<Button
		{...childProps}
		{href}
		size="sm"
		variant="outline"
		aria-label={ariaLabel}
	>
		{#if title}
			<div>{title}</div>
		{/if}
		<Icon {...iconProps} />
	</Button>
{/snippet}

<svelte:window onkeydown={handleKeydown} />
<div
	class="flex w-full flex-row items-center justify-between space-x-4"
	aria-label="top navigation bar"
>
	<BGrp.Root>
		{@render NavButton(
			BASE_HOST_URL,
			House,
			undefined,
			undefined,
			undefined,
			"return to home page"
		)}
		<Dropdown.Root>
			<Dropdown.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						size="sm"
						variant="outline"
						class="ps-2!"
						aria-label="open leaderboard menu"
					>
						<div>leaderboards</div>
						<ChevronDown />
					</Button>
				{/snippet}
			</Dropdown.Trigger>
			<Dropdown.Content align="start" class="mt-2 w-56 font-iosevka">
				<Dropdown.Group>
					<Dropdown.Label class="font-bold text-muted-foreground/75"
						>leaderboards</Dropdown.Label
					>
					<Dropdown.Separator class="mb-1" />
					<Dropdown.Item class="w-full">
						<a
							href={routeUrl("channel").href}
							class="flex w-full flex-row items-center justify-between px-2"
						>
							<div>channels</div>
							<Tv />
						</a>
					</Dropdown.Item>
					<Dropdown.Item class="w-full">
						<a
							href={routeUrl("chatter").href}
							class="flex w-full flex-row items-center justify-between px-2"
						>
							<div>chatters</div>
							<UsersRound />
						</a>
					</Dropdown.Item>
				</Dropdown.Group>
			</Dropdown.Content>
		</Dropdown.Root>
	</BGrp.Root>

	<BGrp.Root class="hidden flex-row items-center justify-center md:flex ">
		{#if inputVisible}
			<div
				in:slide={{ delay: 100, duration: 250, axis: "x", easing: expoOut }}
				out:slide={{ delay: 0, duration: 250, axis: "x", easing: expoOut }}
				class="w-[350px] rounded-full"
			>
				<InputGroup.Root
					class="w-[350px] rounded-full transition-all duration-200 ease-out"
				>
					<SearchInput
						bind:ref={inputRef}
						bind:value={current}
						{loading}
						{current}
						clearSearch={() => {
							current = "";
							previous = "";
						}}
						{results}
					/>
				</InputGroup.Root>
			</div>
		{:else}
			<div
				in:slide={{ delay: 150, duration: 175 }}
				out:slide={{ delay: 0, duration: 75 }}
			>
				<Button
					variant="outline"
					class="rounded-full"
					size="icon-sm"
					onclick={() => (inputVisible = true)}
				>
					<SearchIcon class="size-3.5" />
				</Button>
			</div>
		{/if}
	</BGrp.Root>
	<Dropdown.Root>
		<Dropdown.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					size="sm"
					variant="outline"
					aria-label="miscellaneous navigation pages"
				>
					<EllipsisVerticalIcon />
				</Button>
			{/snippet}
		</Dropdown.Trigger>
		<Dropdown.Content align="end" class="mt-2 w-56 font-iosevka">
			<Dropdown.Group>
				<Dropdown.Label class="font-bold text-muted-foreground/75"
					>:3</Dropdown.Label
				>
				<Dropdown.Separator class="mb-1" />
				<Dropdown.Item class="w-full">
					<a
						href={routeUrl("bot").href}
						class="flex w-full flex-row items-center justify-between px-2"
					>
						<div>chat commands</div>
						<RouterIcon />
					</a>
				</Dropdown.Item>
				<Dropdown.Item class="w-full">
					<a
						href={routeUrl("about").href}
						class="flex w-full flex-row items-center justify-between px-2"
					>
						<div>about</div>
						<CircleQuestionMarkIcon />
					</a>
				</Dropdown.Item>
				<Dropdown.Separator class="mb-1" />
				<Dropdown.Item class="w-full">
					<button
						class="flex w-full flex-row items-center justify-between px-2"
						onclick={toggleModeCookie}
					>
						<div class="flex flex-row justify-between">
							{#if mode.current === "dark"}
								<ToggleRight />
							{:else}
								<ToggleLeft />
							{/if}
						</div>
						<div>dark mode</div>
						<!-- <div> -->
						<!-- 	<ModeChanger /> -->
						<!-- </div> -->
					</button>
				</Dropdown.Item>
			</Dropdown.Group>
		</Dropdown.Content>
	</Dropdown.Root>
</div>
