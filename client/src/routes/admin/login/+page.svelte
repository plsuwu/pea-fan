<script lang="ts">
	import type { PageProps } from "./$types";
	import { goto } from "$app/navigation";
	import { enhance } from "$app/forms";
	import { REGEXP_ONLY_DIGITS } from "bits-ui";
	import { Button } from "$lib/shadcn-components/ui/button";
	import * as InputOTP from "$lib/shadcn-components/ui/input-otp";
	import { KeyRoundIcon } from "@lucide/svelte";

	let { form }: PageProps = $props();

	let failed: string | undefined = $state();
	let waiting = $state(false);
	let successful = $derived(form && form.is_valid === true);

	$effect(() => {
		if (successful && !waiting) {
            waiting = false;
			goto("/admin");
		}

		if (successful === false && !waiting) {
			failed = "invalid token";
		}
	});
</script>

<div class="flex flex-col items-center justify-center px-10">
	<div class="mb-14 self-center text-start text-2xl">login</div>
	<form
		method="POST"
		use:enhance={() => {
			waiting = true;
			return async ({ update }) => {
				await update({ reset: true, invalidateAll: false });
				waiting = false;
			};
		}}
	>
		<div class="flex flex-col items-center">
			<InputOTP.Root maxlength={6} pattern={REGEXP_ONLY_DIGITS} name="token">
				{#snippet children({ cells })}
					<InputOTP.Group>
						{#each cells.slice(0, 3) as cell}
							<InputOTP.Slot {cell} />
						{/each}
					</InputOTP.Group>
					<InputOTP.Separator />
					<InputOTP.Group>
						{#each cells.slice(3, 6) as cell}
							<InputOTP.Slot {cell} />
						{/each}
					</InputOTP.Group>
				{/snippet}
			</InputOTP.Root>
			<Button
				type="submit"
				size="icon-sm"
				variant="outline"
				class="mt-4 self-end rounded-full"
			>
				<KeyRoundIcon class="size-3.5 self-center" />
			</Button>
		</div>
		{#if failed}
			<div class="mt-8 text-end text-red-600">
				{failed}
			</div>
		{/if}
	</form>
</div>
