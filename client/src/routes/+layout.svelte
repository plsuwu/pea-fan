<script lang="ts">
	import '../app.css';

	import { ModeWatcher } from 'mode-watcher';
	import Navbar from '$lib/components/navbar/Navbar.svelte';
	import SidebarLayout from '$lib/components/sidebar/SidebarLayout.svelte';

	let { data, children } = $props();
</script>

<ModeWatcher />
<div class="text-foreground flex w-full flex-col items-center justify-center">
	<Navbar {data} />
	<div class="flex w-full flex-row">
		<SidebarLayout {data} />
		{@render children()}
	</div>
</div>
