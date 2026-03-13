<script lang="ts">
	import { enhance } from "$app/forms";
	import { GitMergeIcon } from "@lucide/svelte";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import ButtonGroup from "./button-group.svelte";
	import Current from "./current.svelte";
	import Fields from "./fields.svelte";
	import Historic from "./historic.svelte";
	import ListItem from "./list-item.svelte";

	let {
		waiting,
		setWaiting,
		formActionResult,
	}: {
		waiting: boolean;
		setWaiting: (state: boolean) => void;
		formActionResult: unknown;
	} = $props();

	let userType = $state<"channel" | "chatter">("channel");
	let current = $state<string>("");

	let aliases = $state<string[]>([]);
	let aliasInputState = $state<string>("");

	function addAlias() {
		if (aliasInputState && !aliases.includes(aliasInputState)) {
			let input: string[] = [aliasInputState];
			if (aliasInputState.includes(",")) {
				input = aliasInputState
					.split(",")
					.filter((str) => str !== "")
					.map((str) => str.trim());
			}
			aliasInputState = "";
			aliases = [...aliases, ...input];
		}
	}

	function removeAlias(item: string) {
		aliases = aliases.filter((alias) => alias !== item);
	}
</script>

<form
	class="w-[435px]"
	method="POST"
	action="?/update"
	use:enhance={({ formData }) => {
		setWaiting(true);

		formData.set("type", userType);
		formData.set("current", current);
		formData.set("historic", JSON.stringify(aliases));

		aliases = [];
		return async ({ update, result }) => {
			await update({ reset: true, invalidateAll: true });
			formActionResult = result;
			setWaiting(false);
		};
	}}
>
	<Fields>
		<Current bind:current bind:userType {waiting} />
		<Historic bind:aliasInputState {addAlias} {waiting} />
	</Fields>
	<div class="mt-8 mb-12 flex w-[350px] flex-row justify-end self-end">
		<ButtonGroup {aliases} {current} {waiting} />
	</div>
</form>

<form
	class="flex w-[350px] flex-row justify-end"
	method="POST"
	action="?/merge"
	use:enhance={() => {
		setWaiting(true);

		return async ({ update, result }) => {
			await update({ reset: true, invalidateAll: true });
			formActionResult = result;
			setWaiting(false);
		};
	}}
>
	<Button
		class="flex flex-row items-center justify-between space-x-3 self-end 
        text-muted-foreground/55 hover:text-muted-foreground"
		disabled={waiting}
		variant="outline"
		type="submit"
	>
		<div>run migration</div>
		<GitMergeIcon />
	</Button>
</form>

{#if aliases.length > 0}
	<div class="mt-8 flex w-full flex-col px-4">
		<div class="mb-4 text-muted-foreground/55">
			{#if current}
				<div>will merge:</div>
			{:else}
				<span
					>(requires specifying a
					<kbd class="rounded-lg bg-accent px-2">current</kbd>
					login to perform merge)
				</span>
			{/if}
		</div>
		<div>
			{#each aliases as alias}
				<ListItem
					{current}
					{removeAlias}
					{userType}
					{waiting}
					itemText={alias}
				/>
			{/each}
		</div>
	</div>
{/if}
