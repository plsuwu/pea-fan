<script lang="ts">
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import { expoOut } from "svelte/easing";
	import { fade } from "svelte/transition";

    let { waiting } = $props();
</script>

{#if waiting}
	<div
		in:fade={{ duration: 250, easing: expoOut }}
		out:fade={{ duration: 100, easing: expoOut }}
		class="fixed top-0 left-0 z-20 h-full w-full bg-background/80 backdrop-blur-sm"
	>
		<div
			class="relative flex h-full w-full flex-row items-center justify-center"
		>
			<Spinner class="relative size-6" />
		</div>
	</div>
{/if}
