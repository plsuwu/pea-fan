import type { LayoutServerLoad } from './$types';
import RedisClient from '$lib/server/db/redis';
import type { Chatter } from '$lib/types';

export const load: LayoutServerLoad = async ({ locals, fetch }) => {
	await RedisClient.migrateOldData();

	let leaderboard: { channel: string; leaderboard: any[] } | null = null;
	let live = null;

	if (locals.currentChannel) {
		leaderboard = {
			channel: locals.currentChannel,
			leaderboard: await RedisClient.getLeaderboard(
				'channel',
				'login',
				locals.currentChannel
			)
		};
	} else {
		const liveQuery = await fetch('/api/socket-query');
		live = await liveQuery.json();
		live = live.data;
	}

	const chatters = await RedisClient.getChatters();
	const channels = await RedisClient.getChannels();

	if (live && live.length > 0) {
		const livenames = live.map((l: any) => l.user_login);
		channels.forEach((channel, index) => {
			if (livenames.includes(channel.broadcaster)) {
				const idx = livenames.indexOf(channel.broadcaster);
                live[idx].rank = index + 1;
				live[idx].image = channel.image;
			}
		});
	}

    // live.push(...live);
    // live.push(...live);
    // live.push(...live);

	return {
		chatters,
		channels,
		live,
		leaderboard
	};
};
