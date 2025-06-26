<script lang="ts">
	import Web from './icons/Web.svelte';
	import Github from './icons/Github.svelte';
	import Twitch from './icons/Twitch.svelte';

	const links = ['https://github.com/plsuwu', 'https://plsuwu.com', 'https://tiles.plsuwu.com'];
</script>

<div class="font-display mt-12 flex w-full flex-col items-end overflow-x-hidden">
	<div class="w-full p-4 text-end text-sm italic">
		<p>
			<span class="font-bold">Note:</span> counter only detects chat messages when a channel is live.
		</p>
		<p class="text-xs">
			tracking since <span class="not-italic font-semibold">Jun 10, 2025</span> (or slightly later).
		</p>
		<p class="mt-16 text-xs">made mostly with hate by pls</p>
	</div>
	<div class="flex-row text-xs">
		<div class="flex flex-row justify-between space-x-3 overflow-x-hidden px-2 pb-4">
			<a href="https://github.com/plsuwu" target="_blank" class="px-1 hover:brightness-50"
				><Github size={15} color={'#a3004c'} />
			</a>
			<a href="https://plsuwu.com" target="_blank" class="px-1 hover:brightness-50"
				><Web size={15} color={'#a3004c'} />
			</a>
			<a href="https://tiles.plsuwu.com" target="_blank" class="px-1 hover:brightness-50"
				><Web size={15} color={'#a3004c'} />
			</a>
			<a href="https://twitch.tv/plss" target="_blank" class="px-1 hover:brightness-50"
				><Twitch size={15} color={'#a3004c'} />
			</a>
		</div>
	</div>
</div>
