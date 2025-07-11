<script lang="ts">
	import Tabs from './options/Tabs.svelte';

	let { data } = $props();

	let mutableData = $state(data);

	let loading = $state(false);
	let hasMoreContent = $state({ channels: true, chatters: true });

	async function onContinueLoad(key: 'user' | 'channel') {
		loading = true;

		let offset =
			key === 'user'
				? mutableData.chatters.length
				: mutableData.channels.length;


		let uri = `/api/cache-query?key=${key}&offset=${offset}`;
        if (data.leaderboard && data.leaderboard.channel) {
            uri += `&user=${data.leaderboard.channel}`;
        }
        console.log(uri);
        

		let res = await fetch(uri, {
			method: 'GET'
		});

		console.log(res.status, res.statusText);
		if (res.status == 200) {

			let body = await res.json();
			if (body === null) {
				key === 'user'
					? (hasMoreContent.chatters = false)
					: (hasMoreContent.channels = false);
			} else {
				if (key === 'user') {
					mutableData.chatters.push(...body);
				} else {
					mutableData.channels.push(...body);
				}
			}
		}

		loading = false;
	}
</script>

<aside
	class="inline-flex h-[calc(100vh-60px)] w-[350px] shrink-0 flex-col justify-between overflow-hidden border-r py-3"
>
	<Tabs
		{hasMoreContent}
		data={mutableData}
		onContinueLoad={(key) => onContinueLoad(key)}
		{loading}
	/>
	<div class="inline-flex w-full flex-col px-4">
		<div class="text-[12px]">don't want your username displayed?</div>
		<a
			class="hover:text-foreground-alt w-fit text-[12px] italic underline transition-colors duration-150 ease-out"
			href="/redact">redact your username here.</a
		>
	</div>
</aside>
