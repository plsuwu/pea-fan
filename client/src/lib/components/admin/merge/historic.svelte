<script lang="ts">
	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import { PlusIcon } from "@lucide/svelte";

	let {
		aliasInputState = $bindable(),
		addAlias,
		waiting
	}: {
		aliasInputState?: string;
		addAlias: () => void;
		waiting: boolean;
	} = $props();
</script>

<InputGroup.Root class="max-w-[350px] rounded-full">
	<InputGroup.Input
		placeholder="pls_, plsssss, ..."
		class="px-4 placeholder:text-muted-foreground/50"
		bind:value={aliasInputState}
		disabled={waiting}
	/>
	<InputGroup.Addon align="inline-end">
		<InputGroup.Button
			onclick={() => addAlias()}
			variant="outline"
			size="icon-xs"
			class="rounded-full"
			disabled={waiting}
		>
			<PlusIcon />
		</InputGroup.Button>
	</InputGroup.Addon>
</InputGroup.Root>
