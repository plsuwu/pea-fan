<script lang="ts">
	import '../app.css';

	import { ModeWatcher } from 'mode-watcher';
	import Navbar from '$lib/components/navbar/Navbar.svelte';

	let { children } = $props();

</script>

<ModeWatcher />
<div class="flex w-full flex-col items-center justify-center text-foreground">
	<Navbar />
	{@render children()}
</div>
