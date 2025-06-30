import type { LayoutServerLoad } from './$types';
import { getAllKeyTotals, getBatchedUserImage, getUserImage } from '$lib/server/db/redis';

export const load: LayoutServerLoad = async ({ locals }) => {
    console.log(locals);

	// this should likely be replaced with a serverside `fetch` call to cache the data
	let chatters = await Promise.all((await getAllKeyTotals('user', 'login')).map(async (user) => {
        // const image = await getUserImage(user.login as string);
        return {
            ...user
        }
    }));

	const channels = await Promise.all(
		(await getAllKeyTotals('channel', 'broadcaster')).map(async (chan) => {
			const broadcaster = (chan.broadcaster as string).split('#')[1];
			const image = await getUserImage(broadcaster);
			return {
				broadcaster,
				image,
				total: chan.total
			};
		})
	);

	return {
		chatters,
		channels
	};
};
