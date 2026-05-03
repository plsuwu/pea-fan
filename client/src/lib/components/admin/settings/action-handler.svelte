<script lang="ts">
	import { enhance } from "$app/forms";
	import Label from "$lib/shadcn-components/ui/label/label.svelte";

	let { setWaiting, action, label, children } = $props();
</script>

<form
	method="POST"
	{action}
	class="flex flex-col items-center space-y-2"
	use:enhance={() => {
		setWaiting(true);

		return async ({ update }) => {
			await update();
			setWaiting(false);
		};
	}}
>
	<div class="flex w-full flex-row md:justify-between">
		<Label for="bot-id" class="mr-4 pl-8 text-xs md:text-sm">{label}</Label>
		<div class="flex flex-row items-center">
			{@render children?.()}
		</div>
	</div>
</form>
