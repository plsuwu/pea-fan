<script lang="ts">
	import type { PageProps } from "./$types";
	import Spinner from "$lib/shadcn-components/ui/spinner/spinner.svelte";
	import { fade } from "svelte/transition";
	import { expoInOut } from "svelte/easing";
	import Form from "$lib/components/admin/merge/form.svelte";

	let { form }: PageProps = $props();

	let waiting = $state(false);
	let formActionResult = $state<any>(null);

	function setWaiting(state: boolean) {
		waiting = state;
	}
</script>

<div class="flex w-full flex-col justify-center px-10">
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
