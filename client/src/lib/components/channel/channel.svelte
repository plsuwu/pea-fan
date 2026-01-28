<script lang="ts">
	import type { ChannelEntry } from "$lib/types";

	let { channel, leaderboard }: { channel: string; leaderboard: ChannelEntry } =
		$props();
</script>

<div class="w-full">
    <div class="text-4xl text-center">
        {channel}
    </div>
</div>
