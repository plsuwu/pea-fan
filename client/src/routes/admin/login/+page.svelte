<script lang="ts">
	import * as InputGroup from "$lib/shadcn-components/ui/input-group";
	import { Spinner } from "$lib/shadcn-components/ui/spinner";
	import { KeyRoundIcon } from "@lucide/svelte";
	import type { PageProps } from "./$types";
	import { goto } from "$app/navigation";

	let { data, form }: PageProps = $props();

	let waiting = $state(false);
	let authFailure = $derived(form && form?.verified !== true);
	let authSuccess = $derived(form?.verified === true);

	$effect(() => {
		if (waiting && form != null) {
			waiting = false;
		}

		if (authSuccess) {
			goto("/admin");
		}
	});

	function handleSubmit() {
		waiting = true;
	}
</script>

<div class="flex flex-row items-center px-10">
	<form method="POST" action="?/login" onsubmit={handleSubmit}>
		<div class="w-1/2 flex-col">
			{#if form != null && authFailure}
				<div class="px-2 py-2 text-red-500">invalid</div>
			{:else}
				<div class="h-[40px]"></div>
			{/if}
			<InputGroup.Root>
				<InputGroup.Input
					type="password"
					placeholder="token"
					name="token"
					class="h-[100px] w-[450px]"
					disabled={waiting}
				/>
				<InputGroup.Addon align="inline-end">
					<InputGroup.Button
						variant="outline"
						class="rounded-full"
						size="icon-xs"
						type="submit"
						disabled={waiting}
					>
						{#if waiting}
							<Spinner />
						{:else}
							<KeyRoundIcon class="p-px" />
						{/if}
					</InputGroup.Button>
				</InputGroup.Addon>
			</InputGroup.Root>
		</div>
	</form>
</div>
