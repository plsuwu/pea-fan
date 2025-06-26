<script lang="ts">
	import Footer from '$lib/components/Footer.svelte';
	import '../app.css';

	let { children } = $props();
</script>

<div
	class="flex min-h-screen w-full flex-col items-center justify-between overflow-x-hidden subpixel-antialiased"
>
	<div class="flex h-screen w-full max-w-full flex-1 flex-col items-center justify-center">
		{@render children()}
	</div>
	<Footer />
</div>
