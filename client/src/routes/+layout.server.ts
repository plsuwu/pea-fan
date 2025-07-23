import type { LayoutServerLoad } from './$types';
import { _migrateOldFormat } from '$lib/server/db/psql/migrate';

export const load: LayoutServerLoad = async ({ locals, fetch }) => {
    // console.log('migrating...');
    // _migrateOldFormat();

	// let leaderboard: { channel: string; leaderboard: any[] } | null = null;
	// let live = null;
	//
	// if (locals.currentChannel) {
	// 	leaderboard = {
	// 		channel: locals.currentChannel,
	// 		leaderboard: await RedisHandler.getLeaderboard(
	// 			'channel',
	// 			locals.currentChannel
	// 		)
	// 	};
	// } else {
	// 	const liveQuery = await fetch('/api/socket-query');
	// 	live = await liveQuery.json();
	// 	live = live.data;
	// }
	//
	// // const chatters = await RedisHandler.getChatters();
	// const channels = await RedisHandler.getChannels();
	//
	// if (live && live.length > 0) {
	// 	const livenames = live.map((broadcaster: any) => broadcaster.user_login);
	// 	channels.forEach((channel, index) => {
	// 		if (livenames.includes(channel.login)) {
	// 			const idx = livenames.indexOf(channel.login);
	//                live[idx].rank = index + 1;
	// 			live[idx].image = channel.image;
	// 		}
	// 	});
	// }
	//
	//    let current = locals.currentChannel;
	// return {
	// 	chatters: [],
	// 	channels,
	// 	live,
	// 	leaderboard,
	//        current,
	// };
};
