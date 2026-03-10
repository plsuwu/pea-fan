<script lang="ts">
	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import * as Dropdown from "$lib/shadcn-components/ui/dropdown-menu";
	import { ChevronDownIcon } from "@lucide/svelte";

	let {
		current = $bindable(),
		userType = $bindable(),
		waiting
	}: {
		current: string;
		userType: "channel" | "chatter";
		waiting: boolean;
	} = $props();
</script>

{#snippet InputTypeDropdown(
	options: Array<"channel" | "chatter"> = ["channel", "chatter"]
)}
	<Dropdown.Root>
		<Dropdown.Trigger>
			{#snippet child({ props })}
				<InputGroup.Button
					{...props}
					variant="outline"
					class="rounded-full pe-1.5! text-sm font-semibold text-muted-foreground"
					>{userType}<ChevronDownIcon class="size-3" />
				</InputGroup.Button>
			{/snippet}
		</Dropdown.Trigger>
		<Dropdown.Content align="end">
			<Dropdown.RadioGroup bind:value={userType}>
				{#each options as option}
					<Dropdown.RadioItem value={option}>{option}</Dropdown.RadioItem>
				{/each}
			</Dropdown.RadioGroup>
		</Dropdown.Content>
	</Dropdown.Root>
{/snippet}

<InputGroup.Root class="max-w-[350px] rounded-full">
	<InputGroup.Input
		bind:value={current}
		disabled={waiting}
		placeholder="plss"
		class="max-w-[350px] px-4 placeholder:text-muted-foreground/50"
		id="current"
		name="current"
	/>
	<InputGroup.Addon align="inline-end">
		{@render InputTypeDropdown()}
	</InputGroup.Addon>
</InputGroup.Root>
