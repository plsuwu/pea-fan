<script lang="ts">
	import Footer from '$lib/components/Footer.svelte';
	import '../app.css';

	let { children } = $props();
</script>

<div
	class="min-h-screen flex w-screen flex-col items-center justify-between overflow-x-hidden overflow-y-scroll"
>
	<div class="flex h-screen w-full flex-1 flex-col items-center justify-center">
		{@render children()}
	</div>
	<Footer />
</div>
