<script lang="ts">
	import { cn } from "$lib/shadcn-components/utils.js";
    import { LoaderIcon } from "@lucide/svelte";
	import type { ComponentProps } from "svelte";

	let { class: className, ...restProps }: ComponentProps<typeof LoaderIcon> =
		$props();
</script>

<LoaderIcon
	role="status"
	aria-label="Loading"
	class={cn("size-4 animate-spin", className)}
	{...restProps}
/>
