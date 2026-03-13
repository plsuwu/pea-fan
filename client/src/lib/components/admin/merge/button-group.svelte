<script lang="ts">
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import { CombineIcon } from "@lucide/svelte";

	let {
		waiting,
		current,
		aliases
	}: { waiting: boolean; current?: string; aliases: string[] } = $props();

	let disabled = $derived(
		waiting || !current || current === "" || aliases.length === 0
	);
</script>

<Button
	variant="outline"
	type="submit"
	class="flex flex-row items-center justify-between space-x-3 self-end 
        text-muted-foreground/55 hover:text-muted-foreground"
	{disabled}
>
    <div>
        merge
    </div>
	<CombineIcon />
</Button>
