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
	const BASE_API_URL = `${Rh.proto}://${Rh.api}`;

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
				const url = new URL(`${BASE_API_URL}/search/by-login`);
				url.searchParams.set("login", login);

				return url;
			};

			const res = await fetch(queryUrl().href, { signal });
			if (!signal.aborted) {
				const body = await res.json();
				results = body[0];
				count = body[1];
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
	let count = $state(0);

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
		faq: "/faq",
	} as const;

	type Route = keyof typeof ROUTE;

	function routeUrl(route: Route) {
		return new URL(BASE_HOST_URL + ROUTE[route]);
	}

	function handleKeydown(event: KeyboardEvent) {
		const { key, ctrlKey } = event;
		if (key === "j" && ctrlKey) {
			event.preventDefault();
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
	iconProps?: Record<string, unknown>
)}
	<Button {...childProps} {href} size="sm" variant="outline">
		{#if title}
			<div>{title}</div>
		{/if}
		<Icon {...iconProps} />
	</Button>
{/snippet}

<svelte:window onkeydown={handleKeydown} />
<div class="flex w-full flex-row items-center space-x-4">
	<BGrp.Root>
		{@render NavButton(BASE_HOST_URL, House)}
		<Dropdown.Root>
			<Dropdown.Trigger>
				{#snippet child({ props })}
					<Button {...props} size="sm" variant="outline" class="ps-2!">
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

	<BGrp.Root class="hidden flex-row items-center justify-center md:flex">
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
</div>

<BGrp.Root>
	<Dropdown.Root>
		<Dropdown.Trigger>
			{#snippet child({ props })}
				<Button {...props} size="sm" variant="outline" class="ps-2!">
					<MenuIcon />
				</Button>
			{/snippet}
		</Dropdown.Trigger>
		<Dropdown.Content align="end" class="mt-2 w-56 font-iosevka">
			<Dropdown.Group>
				<!-- <Dropdown.Label class="font-bold text-muted-foreground/75"> -->
				<!-- 	things -->
				<!-- </Dropdown.Label> -->
				<Dropdown.Item class="w-full">
					<a
						href={routeUrl("faq").href}
						class="flex w-full flex-row items-center justify-between px-2"
					>
						<div>about</div>
						<CircleQuestionMarkIcon />
					</a>
				</Dropdown.Item>
				<Dropdown.Separator class="mb-1" />
				<Dropdown.Item class="w-full">
					<button
						class="flex w-full flex-row items-center justify-center px-2"
						onclick={toggleModeCookie}
					>
						<div></div>
						<div>
							<ModeChanger />
						</div>
					</button>
				</Dropdown.Item>
			</Dropdown.Group>
		</Dropdown.Content>
	</Dropdown.Root>
</BGrp.Root>
