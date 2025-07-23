import type { RequestEvent } from '@sveltejs/kit';
import { json } from '@sveltejs/kit';

export const GET = async (event: RequestEvent) => {
	const params = event.url.searchParams;

	const key = params.get('key') ?? null;
	const offset = Number(params.get('offset')) ?? null;
	const chatter = params.get('chatter') ?? 'global';

	if (!key || !['chatter', 'channel'].includes(key) || offset == null) {
		return json({
			status: 400,
			statusText: "Missing or invalid key or offset"
		});
	}

	return json('UNAVAILABLE', { status: 200 });
};
