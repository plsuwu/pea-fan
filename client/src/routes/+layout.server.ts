import type { LayoutServerLoad } from './$types';
import RedisClient from '$lib/server/db/redis';

export const load: LayoutServerLoad = async ({ locals }) => {

	// this should likely be replaced with a serverside `fetch` call to cache the data, i think?
	const chatters = await RedisClient.getChatters();
	const channels = await RedisClient.getChannels();

	return {
		chatters,
		channels
	};
};
