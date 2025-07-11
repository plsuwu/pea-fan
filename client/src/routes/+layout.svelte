<script lang="ts">
	import '../app.css';

	import { ModeWatcher } from 'mode-watcher';
	import Navbar from '$lib/components/navbar/Navbar.svelte';
	import SidebarLayout from '$lib/components/sidebar/SidebarLayout.svelte';

	let { data, children } = $props();

	let pageTitle: string = $state('piss leaderboard');
	if (data.leaderboard && data.leaderboard.channel) {
		pageTitle = `${data.leaderboard.channel}'s piss stats`;
	}
</script>

<svelte:head>
	<title>{pageTitle}</title>
</svelte:head>

<ModeWatcher />
<div class="text-foreground flex w-full flex-col items-center justify-center">
	<div class="sticky left-0 top-0 z-10 w-full bg-background/90">
		<Navbar {data} />
	</div>
	<div class="flex w-full flex-row">
		<div class="sticky top-[60px] h-full">
			<SidebarLayout {data} />
		</div>
		<div class="w-full h-full">
			{@render children()}
		</div>
	</div>
</div>
