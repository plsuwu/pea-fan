<script lang="ts">
	import { page } from "$app/state";
	import Nav from "$lib/components/admin/nav/nav.svelte";

	let { children } = $props();
</script>

<div class="-mt-12 h-full w-full lg:-mx-11">
	{#if page.url.pathname !== "/admin/login"}
		<Nav />
	{/if}
	<div class="mb-12"></div>
	{@render children?.()}
</div>
