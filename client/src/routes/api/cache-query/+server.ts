import type { RequestEvent } from '@sveltejs/kit';
import type { KeyReturnType, KeyStoredType } from '$lib/server/db/redis';
import { json } from '@sveltejs/kit';
import RedisClient from '$lib/server/db/redis';

export const GET = async (event: RequestEvent) => {
	const params = event.url.searchParams;

	const key = params.get('key') ?? null;
	const offset = Number(params.get('offset')) ?? null;
	const user = params.get('user') ?? 'global';

	if (!key || !['user', 'channel'].includes(key) || !offset) {
		return json({
			status: 400,
			statusText: "missing or invalid 'key' or 'offset' specifier"
		});
	}

	const returnKey = key === 'user' ? 'login' : 'broadcaster';
	let query = await RedisClient.getLeaderboard(
		key as KeyStoredType,
		returnKey as KeyReturnType,
        user,
		offset
	);

	if (query.length === 0) {
		return json(null, { status: 200 });
	}

	return json(query, { status: 200 });
};
