<script lang="ts">
	import type { PageProps } from './$types';
	let { data }: PageProps = $props();

	let leaderboard = data.body.leaderboard;
	let broadcaster = data.broadcaster;

	function getPlural(num: number) {
		if (num != 1) {
			return 'times';
		}
		return 'time';
	}
</script>

<div class="flex h-full w-full flex-col items-center justify-center">
	<div class="flex w-1/5 flex-col items-center justify-center self-center">
		<div class="flex flex-col self-start text-2xl font-medium">
			<!-- im not fucking recompiling the rust server tonight so you get a hardcoded color for now DEAL WITH IT deal with it....-->
			<div>
				<span
					class="duyration-150 font-semibold italic text-[#9FD7D9] transition-opacity ease-in-out hover:opacity-50"
					><a href={`https://twitch.tv/${broadcaster}`}>{broadcaster}'s</a></span
				> top piss mentioners:
			</div>
		</div>
		{#each leaderboard as leader}
			<div class="flex w-full flex-row place-content-between space-x-3">
				<div><span class="font-semibold italic">{leader[0]}</span> has said piss</div>
				<div>{leader[1]} {getPlural(leader[1])}</div>
			</div>
		{/each}
	</div>
</div>
