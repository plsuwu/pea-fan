<script lang="ts">
	import type { Snippet } from "svelte";

	let { children, ...rest }: { children: Snippet; [key: string]: unknown } =
		$props();
</script>

<kbd class="rounded-full bg-accent-foreground/15 px-2 py-0" {...rest}>
	{@render children()}
</kbd>
