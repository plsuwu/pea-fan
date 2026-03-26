<script lang="ts">
	import type { PageProps } from "./$types";
	import { fade } from "svelte/transition";
	import { expoInOut } from "svelte/easing";
	import { readableColor } from "$lib/utils";
	import { Checkbox } from "$lib/shadcn-components/ui/checkbox";
	import { Button } from "$lib/shadcn-components/ui/button";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import Form from "$lib/components/admin/merge/form.svelte";
	import { mode } from "mode-watcher";
	import { enhance } from "$app/forms";
	import type { SearchResult } from "$lib/utils/search";
	import Input from "$lib/shadcn-components/ui/input/input.svelte";
	import * as InputGroup from "$lib/shadcn-components/ui/input-group/index";
	import {
		ArrowBigRightDashIcon,
		ArrowRightIcon,
		ListXIcon,
		UndoIcon,
	} from "@lucide/svelte";

	type ChannelData = {
		id: string;
		enabled: string;
		name: string;
		login: string;
		color: string;
		image: string;
	};

	let { data, form }: PageProps = $props();

	let waiting = $state(false);
	let formActionResult = $state<any>(null);

	let channels: ChannelData[] | null = $derived.by(() =>
		data.channels == null
			? []
			: data.channels.sort((a, b) => a.login.localeCompare(b.login))
	);
	let channelFilterString: undefined | string = $state();
	let filteredChannels = $derived.by(() => {
		if (!channelFilterString || channelFilterString === "") {
			return channels;
		} else if (channels != null) {
			const res = channels.filter((entry: ChannelData) => {
				return entry.login.includes(channelFilterString!.toLowerCase());
			});

			return res;
		} else {
			[];
		}
	});

	let clearByIdInput: string | undefined = $state();
	let searchInput: string | undefined = $state();
	let searchResult: undefined | { count: number; results: SearchResult[] } =
		$derived.by(() => {
			if (form && form.from === "searchChatter") {
				$inspect(form);
				const results = form.results[0] as SearchResult[];
				const count = form.results[1];

				return { count, results };
			}
		});

	let clearScoreId: string | undefined = $state();

	function setWaiting(state: boolean) {
		waiting = state;
	}
</script>

{#snippet SearchResult(title: string, content: string)}
	<div class="flex flex-row justify-between">
		<div>{title}</div>
		<div>{content}</div>
	</div>
{/snippet}

<div class="flex flex-col overflow-hidden pr-6">
	<div class="flex flex-col">
		<div class="flex w-full flex-col px-10">
			<Form {waiting} {setWaiting} {formActionResult} />
		</div>

		{#if formActionResult}
			{#if formActionResult.success !== true}
				<div class="m-8 flex w-[350px] items-center justify-center">
					<div
						class="w-full rounded-md border border-red-900 text-center text-red-500"
					>
						an error occurred
					</div>
				</div>
			{:else if formActionResult.status}
				<div class="m-8 flex w-[350px] items-center justify-center">
					completed with status: <kbd class="rounded-xl bg-muted px-4"
						>{formActionResult.status}</kbd
					>
				</div>
			{/if}
		{/if}
	</div>
	<div class="mt-16 flex flex-col">
		<div class="mb-6 text-2xl">user search (database)</div>
		<form
			method="POST"
			action="?/searchChatter"
			use:enhance={({ formData }) => {
				waiting = true;
				formData.set("query", String(searchInput));

				return async ({ update }) => {
					await update({ reset: false, invalidateAll: false });
					waiting = false;
				};
			}}
		>
			<InputGroup.Root class="rounded-full p-px">
				<InputGroup.Input
					type="text"
					name="search-input"
					class="placeholder:text-muted-foreground/55"
					placeholder="chatter login"
					bind:value={searchInput}
				/>
				<InputGroup.Addon align="inline-end">
					{#if searchResult}
						<Button
							variant="outline"
							size="icon-sm"
							class="size-6 rounded-full"
							onclick={(event) => {
								event.preventDefault();
								searchResult = undefined;
								searchInput = undefined;
							}}
						>
							<UndoIcon />
						</Button>
					{:else}
						<Button
							variant="outline"
							size="icon-sm"
							class="size-6 rounded-full"
							type="submit"
						>
							<ArrowRightIcon />
						</Button>
					{/if}
				</InputGroup.Addon>
			</InputGroup.Root>
		</form>

		{#if searchResult}
			<div class="my-4 max-h-[350px] overflow-y-scroll rounded-lg border p-3">
				<div class="border-b py-2 text-end text-sm font-semibold">
					total results: {searchResult.count}
				</div>
				{#each searchResult.results as result}
					<div class="flex flex-col space-y-1 border-b py-3">
						{@render SearchResult("display_name", result.name)}
						{@render SearchResult("login", result.login)}
						{@render SearchResult("id", result.id)}
						{@render SearchResult("score", result.total.toLocaleString())}
						{@render SearchResult("rank", result.ranking.toLocaleString())}
						<div class="flex flex-row justify-between">
							<div>color</div>
							<div style:color={`${readableColor(result.color)}`}>
								{result.color}
							</div>
						</div>
						<div class="flex flex-row items-center justify-between">
							<div>profile_image</div>
							<div>
								<img
									src={result.image}
									alt={`${result.login}`}
									class="size-12 rounded-full"
								/>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
	<div class="mt-16 flex flex-col">
		<div class="mb-6 text-2xl">clear score (chatter)</div>
		<form
			method="POST"
			action="?/clearScore"
			use:enhance={({ formData }) => {
				waiting = true;
				console.log(formData);

				return async ({ update }) => {
					await update();
					waiting = false;
				};
			}}
		>
			<InputGroup.Root class="rounded-full p-px">
				<InputGroup.Input
					type="text"
					name="id"
					class="placeholder:text-muted-foreground/55"
					placeholder="chatter id"
					bind:value={clearByIdInput}
				/>
				<InputGroup.Addon align="inline-end">
					<Button
						variant="outline"
						size="icon-sm"
						class="size-6 rounded-full"
						type="submit"
						disabled={waiting}
					>
						<ListXIcon />
					</Button>
				</InputGroup.Addon>
			</InputGroup.Root>
		</form>
	</div>
</div>

<div class="flex w-full flex-col">
	<div class="mb-10 text-2xl">channel config</div>
	<div
		class="channel-table max-h-[1000px] min-h-[1000px] rounded-lg border border-accent p-6"
		role="region"
		tabindex="-1"
	>
		<Input
			type="text"
			placeholder="filter by login..."
			bind:value={channelFilterString}
			class="mb-8 w-1/3"
		/>
		<table>
			<thead>
				<tr>
					<th class="w-[85px] self-start text-start">login</th>
					<th class="w-[60px] self-end text-end xl:w-[60px]">id&nbsp;&nbsp;</th>
					<th
						class="collapse w-0 max-w-0 text-start xl:visible xl:w-[150px] xl:max-w-[150px]"
						>display name</th
					>
					<th class="w-[250px] self-start text-start">color</th>
					<th class="w-[100px] text-end">replies</th>
				</tr>
			</thead>
			<tbody class="">
				{#if filteredChannels}
					{#each filteredChannels as channel}
						<tr>
							<td class="w-[85px] self-start text-start">{channel.login}</td>
							<td class="w-[60px] self-end text-end xl:w-[60px]"
								>{channel.id}&nbsp;&nbsp;</td
							>
							<td
								class="collapse w-0 max-w-0 text-start xl:visible xl:w-[150px] xl:max-w-[150px]"
								>{channel.name}</td
							>
							<td
								class="m-px w-[200px] self-start text-center font-bold text-black"
								style={`background-color: ${readableColor(channel.color, mode.current, 2.5)};`}
							>
								{channel.color}
							</td>
							<td>
								<div class="flex w-[100px] flex-row items-center justify-end">
									<form
										method="POST"
										action="?/toggleReply"
										use:enhance={({ formData }) => {
											waiting = true;
											formData.set("id", channel.id);

											return async ({ update }) => {
												await update();
												waiting = false;
											};
										}}
									>
										<Checkbox
											name="replies-enabled"
											checked={channel.enabled as unknown as boolean}
											type="submit"
										/>
									</form>
								</div>
							</td>
						</tr>
					{/each}
				{/if}
			</tbody>
		</table>
	</div>
</div>

{#if waiting}
	<div
		in:fade={{ delay: 50, duration: 300, easing: expoInOut }}
		out:fade={{ delay: 50, duration: 300, easing: expoInOut }}
		class="fixed top-0 left-0 h-full w-full bg-background/80 backdrop-blur-xs"
	>
		<div class="flex h-full items-center justify-center">
			<Spinner size="6" />
		</div>
	</div>
{/if}

<style>
	.channel-table {
		width: 100%;
		overflow-y: scroll;
	}

	.channel-table table {
		height: 100%;
		width: 100%;
		gap: 8px;
		table-layout: auto;
		text-align: left;
	}

	.channel-table th {
		padding-bottom: 2rem;
		color: var(--color-accent);
		font-size: 20px;
		font-weight: 700;
		border-bottom: 1px solid var(--color-muted);
	}

	.channel-table td {
		padding: 2px;
		padding-right: 5px;
		border-bottom: 1px solid var(--color-muted);
	}

	/* .channel-table tr:nth-child(even) td { */
	/* 	background-color: var(--color-muted) 0.7; */
	/* } */

	/* .channel-table tr:nth-child(odd) td { */
	/* 	background-color: #808080; */
	/* 	color: #000000; */
	/* } */
</style>
