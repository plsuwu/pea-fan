<script lang="ts">
	import { type Component } from "svelte";
	import { page } from "$app/state";
	import {
		House,
		Tv,
		UsersRound,
		ChevronDown,
		FileQuestionMark,
		Search,
		SearchIcon,
	} from "@lucide/svelte";
	import { Rh } from "$lib/utils/route";
	import * as Dropdown from "$lib/shadcn-components/ui/dropdown-menu";
	import * as BGrp from "$lib/shadcn-components/ui/button-group";
	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import SearchInput from "./search.svelte";
	import { fade, fly, scale, slide } from "svelte/transition";
	import { expoOut } from "svelte/easing";

	const baseUrl = `${Rh.proto}://${Rh.deriveBase(page.url.host)}`;
	let inputElement: HTMLInputElement | null = $state(null);
	let searchOpen = $state(false);
	// let inputFocused = $state(false);

	const timeoutFocusInput = async (state: boolean, ms = 100) => {
		setTimeout(() => {
			setSearchInputFocused(state);
		}, ms);
	};

	function setSearchInputFocused(state: boolean) {
		if (inputElement) {
			inputElement.focus();
		}
	}

	function onclose() {
		if (searchOpen) searchOpen = false;
	}

	const ROUTE = {
		channel: "/leaderboard/channel",
		chatter: "/leaderboard/chatter",
	} as const;

	type Route = keyof typeof ROUTE;

	function routeUrl(route: Route) {
		return new URL(baseUrl + ROUTE[route]);
	}

	// function isCurrentRoute(href: string) {
	// 	return href === page.url.pathname;
	// }
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

<div class="flex w-full flex-row items-center space-x-4">
	<BGrp.Root>
		{@render NavButton(baseUrl, House)}
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
					<!-- <Dropdown.Item> -->
					<!-- {@render NavButton(chatters.href, UsersRound)} -->
					<!-- </Dropdown.Item> -->
				</Dropdown.Group>
			</Dropdown.Content>
		</Dropdown.Root>
	</BGrp.Root>

	<BGrp.Root class="md:flex flex-row items-center justify-center hidden">
		<!-- {#if searchOpen} -->
			<div
				in:slide={{ delay: 100, duration: 250, axis: "x", easing: expoOut }}
				out:slide={{ delay: 0, duration: 250, axis: "x", easing: expoOut }}
				class="w-[350px] rounded-full"
			>
				<InputGroup.Root
					class="w-[350px] rounded-full transition-all duration-200 ease-out"
				>
					<SearchInput bind:inputElement {onclose} />
				</InputGroup.Root>
			</div>
		<!-- {:else} -->
		<!-- 	<div -->
		<!-- 		in:fade={{ delay: 150, duration: 175 }} -->
		<!-- 		out:fade={{ delay: 0, duration: 75 }} -->
		<!-- 	> -->
		<!-- 		<Button -->
		<!-- 			variant="outline" -->
		<!-- 			class="rounded-full" -->
		<!-- 			size="icon-sm" -->
		<!-- 			onclick={() => { -->
		<!-- 				searchOpen = !searchOpen; -->
		<!-- 				timeoutFocusInput(true); -->
		<!-- 			}} -->
		<!-- 		> -->
		<!-- 			<SearchIcon class="size-3.5" /> -->
		<!-- 		</Button> -->
		<!-- 	</div> -->
		<!-- {/if} -->
	</BGrp.Root>
</div>

<BGrp.Root>
	<Button variant="outline" size="icon-sm">
		<FileQuestionMark />
	</Button>
</BGrp.Root>
