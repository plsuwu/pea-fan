<script lang="ts">
	import type { PageProps } from "./$types";
	import { fade } from "svelte/transition";
	import { expoInOut } from "svelte/easing";
	import { readableColor } from "$lib/utils";
	import { Checkbox } from "$lib/shadcn-components/ui/checkbox";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import Form from "$lib/components/admin/merge/form.svelte";
	import { mode } from "mode-watcher";
	import { enhance } from "$app/forms";

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
	let channels = $derived(data.channels);

	function setWaiting(state: boolean) {
		waiting = state;
	}

	function handleCheckboxChange() {}
</script>

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

<div class="flex w-full flex-col">
	<div class="mb-10 text-2xl">channel config</div>
	<div
		class="channel-table rounded-lg border border-accent p-6"
		role="region"
		tabindex="-1"
	>
		<table>
			<thead>
				<tr>
					<th>login</th>
					<th class="text-end">id&nbsp;&nbsp;</th>
					<th class="text-start">display name</th>
					<th class="text-start">color</th>
					<th class="text-end">replies</th>
				</tr>
			</thead>
			<tbody class="">
				{#if channels}
					{#each channels.sort( (a, b) => a.login.localeCompare(b.login) ) as channel}
						<tr>
							<td class="max-w-[25px]">{channel.login}</td>
							<td class="max-w-[35px] self-start text-end">{channel.id}&nbsp;&nbsp;</td>
							<td class="max-w-[75px] text-start font-semibold"
								>{channel.name}</td
							>
							<td
								class="m-px text-center font-bold text-black"
								style={`background-color: ${readableColor(channel.color, mode.current, 2.5)};`}
							>
								{channel.color}
							</td>
							<td>
								<div class="flex w-full flex-row items-center justify-end">
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
											checked={channel.enabled}
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
