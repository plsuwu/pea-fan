<script lang="ts">
	import { page } from "$app/state";
	import clientLogger from "$lib/observability/client/logger";

	let error = $derived(page.error);
	let errMessage = $derived(
		error ? error.message : "an unexpected error has occurred"
	);

	$effect(() => {
		if (error) {
			clientLogger.error("error on client", { error: { ...error } });
		}
	});
</script>

<div class="mt-12 flex flex-col items-center w-full">
	<div class="mt-12 mb-2 text-base font-medium text-accent-foreground/55">
		fuck ohhh fuck fuck fuckl SHIT fuckM
	</div>
	<div class="py-4 lg:text-2xl font-bold text-red-400 text-base text-center">{errMessage}</div>

	{#if error}
		<div class="">
			<div class="text-lg text-accent-foreground">
				(status code <span class="font-medium text-red-800">{error.code}</span>)
			</div>
		</div>
	{/if}
</div>
