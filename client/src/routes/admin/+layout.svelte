<script lang="ts">
	import Nav from "$lib/components/admin/nav/nav.svelte";

	let { children } = $props();
</script>

<div class="-mt-12 h-full w-full lg:-mx-11">
	<Nav />
    <div class="mb-12"></div>
	{@render children?.()}
</div>
