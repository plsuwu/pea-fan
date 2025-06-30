<script lang="ts">
	import DarkToggle from '$lib/components/navbar/DarkToggle.svelte';
	import Home from './Home.svelte';
	import Search from './Search.svelte';

	let { data } = $props();
</script>

<div
	class="border-border h-15 flex w-full flex-row items-center justify-between border-b px-8"
>
	<div class="flex flex-row">
		<Home />
		<Search {data} />
	</div>
	<DarkToggle />
</div>
