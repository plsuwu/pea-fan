<script lang="ts">
	import type { Snippet } from "svelte";

	let { children }: { children: Snippet } = $props();
</script>

<div class="mb-4 flex max-w-[350px] w-full flex-col justify-center space-y-4">
	<div class="mb-8 text-2xl">merge</div>
	{@render children()}
</div>
