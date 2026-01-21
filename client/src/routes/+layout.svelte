<script lang="ts">
	import "./layout.css";
	let { children } = $props();
</script>

<svelte:head>
	<title>"a"</title>
</svelte:head>

<div class="flex flex-row items-center justify-center">
	{@render children()}
</div>
