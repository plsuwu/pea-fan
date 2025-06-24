<script lang="ts">
	import DarkToggle from '$lib/components/navbar/DarkToggle.svelte';
	import Home from './Home.svelte';
	import Search from './Search.svelte';
</script>

<div
	class="border-border flex h-15 w-full flex-row items-center justify-between border-b px-8"
>
	<div class="flex flex-row">
		<Home />
		<Search />
	</div>
	<DarkToggle />
</div>
