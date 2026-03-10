<script lang="ts">
	import { MoveLeft, DeleteIcon } from "@lucide/svelte";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";

	let {
		current,
        removeAlias,
		userType,
		waiting,
		itemText
	}: {
		current?: string;
        removeAlias: (item: string) => void;
		userType: "channel" | "chatter";
		waiting: boolean;
		itemText: string;
	} = $props();

	let mergeLine = $derived(
		userType === "channel" ? `#${current}` : `${current}`
	);
</script>

<div
	class="flex w-[350px] flex-row items-center justify-between space-x-4 text-muted-foreground"
>
	<div class="flex flex-row space-x-3">
		{#if current && current !== ""}
			<div class="flex flex-row items-center">
				{mergeLine}
				<MoveLeft class="ml-1 size-4" />
			</div>
		{/if}
		<div>
			{itemText}
		</div>
	</div>
	<Button
		onclick={() => {
			removeAlias(itemText);
		}}
		variant="ghost"
		size="icon-sm"
		class="ml-4 rounded-full p-0"
		disabled={waiting}
	>
		<DeleteIcon class="size-4 text-red-400" />
	</Button>
</div>
