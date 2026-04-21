<script lang="ts">
	import { expoOut } from "svelte/easing";
	import { fade } from "svelte/transition";

	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import * as Kbd from "$lib/shadcn-components/ui/kbd";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";

	import { DeleteIcon, CircleArrowRightIcon } from "@lucide/svelte";
	import { mode } from "mode-watcher";

	import { readableColor } from "$lib/utils";
	import { Rh } from "$lib/utils/route";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";

	let {
		ref = $bindable(),
		value = $bindable(),
		loading,
		current,
		clearSearch,
		results,
	} = $props();

	// const BASE_URL = new URL(`${Rh.proto}://${Rh.api}/search/by-login`);
	function navigateTo(result: (typeof results)[0]) {
		const derivedBase = Rh.deriveBase(page.url.host);
		console.log(page.url.host);

		const href = `${Rh.proto}://${derivedBase}/leaderboard/chatter?page=${result.page}#${result.ranking}`;
		clearSearch();

		if (derivedBase === page.url.host) {
			goto(href);
		} else {
			window.location.href = href;
		}
	}
</script>

<InputGroup.Input
	bind:ref
	bind:value
	type="text"
	class="w-full placeholder:px-2 placeholder:text-muted-foreground/55"
	placeholder="chatter search..."
/>
<InputGroup.Addon align="inline-end">
	{#if loading}
		<div
			in:fade={{ delay: 50, duration: 250, easing: expoOut }}
			out:fade={{ delay: 0, duration: 0 }}
		>
			<Spinner class="shrink-0" />
		</div>
	{:else if !!current && current !== ""}
		<div
			in:fade={{ delay: 250, duration: 500, easing: expoOut }}
			out:fade={{ delay: 0, duration: 0 }}
		>
			<Button
				variant="ghost"
				size="icon-sm"
				class="-mr-2 rounded-full"
				onclick={() => clearSearch()}
			>
				<DeleteIcon />
			</Button>
		</div>
	{:else}
		<div class="items-center">
			<Kbd.Group>
				<Kbd.Root>
					<div class="flex items-center space-x-0 font-iosevka italic">
						<span>{"<"}</span>
						<span>C</span>
						<span class="mx-0.5">-</span>
						<span>j</span>
						<span>{">"}</span>
					</div>
				</Kbd.Root>
			</Kbd.Group>
		</div>
	{/if}
</InputGroup.Addon>

{#if current !== "" && !loading}
	<div class="absolute top-11 z-20 w-full">
		<div
			class="flex h-full w-full flex-col rounded-sm border border-border bg-background text-sm"
		>
			{#if results.length > 0}
				<table>
					<thead>
						<tr class="text-muted-foreground">
							<th
								class="border-b border-b-border px-2 py-0.5 text-start text-base"
								>ranking</th
							>
							<th
								class="border-b border-b-border px-2 py-0.5 text-start text-base"
								>name</th
							>
							<th
								class="border-b border-b-border px-2 py-0.5 text-end text-base"
								>total</th
							>
							<th
								class="border-b border-b-border px-2 py-0.5 text-end text-base"
							></th>
						</tr>
					</thead>
					<tbody>
						{#each results as result}
							{@const colorMode = mode.current === "dark" ? "light" : "dark"}
							<tr
								class="rounded-xl"
								style={`background-color: ${readableColor(result.color, colorMode, 15)};`}
							>
								<td class="px-2 py-px text-start">{result.ranking}</td>
								<td class="px-2 py-px text-start font-bold">{result.name}</td>
								<td class="px-2 py-px text-end">{result.total}</td>
								<td class="w-max px-2 py-px text-end">
									<button
										onclick={() => navigateTo(result)}
										class="flex w-full flex-row justify-end self-end transition-all duration-200 hover:brightness-50"
										aria-label="jump to leaderboard entry"
									>
										<CircleArrowRightIcon size={12} />
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{:else}
				<div>no results</div>
			{/if}
		</div>
	</div>
{/if}
