<script lang="ts">
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import { ChevronRight, ChevronLeft } from "@lucide/svelte";
	import { type Component } from "svelte";
</script>

{#snippet SinglePageButton(Icon: Component)}
	<Button variant="outline">
		<Icon />
	</Button>
{/snippet}

{@render SinglePageButton(ChevronLeft)}
{@render SinglePageButton(ChevronRight)}
