<script lang="ts">
	import { cn } from "tailwind-variants";
	import Hooks from "./hooks.svelte";
	import Toggle from "./toggle.svelte";
	import Name from "./name.svelte";
	import type { Snippet } from "svelte";
	import Id from "./id.svelte";

	interface Props {
		config: any;
		idx: number;
		children?: Snippet;
	}

	let { config, idx, children }: Props = $props();

	let panelState = $state(false);
	function togglePanelState() {
		panelState = !panelState;
	}
</script>

<div
	class={cn(
		"flex w-full flex-col px-4 py-1.5",
		idx % 2 === 0 ? "bg-accent-foreground/10" : ""
	)}
>
	<div
		class="grid w-full grid-cols-1 grid-rows-4 justify-between xl:grid-cols-10
        xl:grid-rows-2"
	>
		<div class="col-start-1 row-start-1 w-full xl:col-span-2">
			<Id {config} />
		</div>
		<div
			class="row-start-2 flex w-full flex-row items-center justify-start space-x-8
            px-4 pt-1 xl:col-span-2 xl:col-start-1"
		>
			<Toggle {panelState} {togglePanelState} />
			<Name {config} />
		</div>
		<div
			class="col-start-3 row-span-2 ml-4 hidden h-full w-max border-collapse
            flex-row space-x-4 border-l border-l-accent-foreground/25 px-2 xl:flex"
		>
			{#each Array({ a: "R", b: config.enabled }, { a: "L", b: config.live }) as state}
				<div
					class="font-semibold italic brightness-25"
					class:option-set={state.b}
				>
					{state.a}
				</div>
			{/each}
		</div>
		<div
			class="row-span-2 row-start-3 xl:col-span-8 xl:col-start-4 xl:row-start-1"
		>
			<Hooks hook={config.hook} />
		</div>
	</div>
</div>
<div
	class:hidden={!panelState}
	class="mt-2 mb-6 rounded-br-lg rounded-bl-lg border-r border-b border-l pb-4"
>
	{@render children?.()}
</div>

<style>
	.option-set {
		filter: brightness(100%);
	}
</style>
