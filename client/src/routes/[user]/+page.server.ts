import RedisClient from '$lib/server/db/redis';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	let user = event.url.pathname.split('/')[1];
    let leaderboard = await RedisClient.getUserLeaderboard('user', user);
	return {
		userLeaderboard: leaderboard
	};
};
