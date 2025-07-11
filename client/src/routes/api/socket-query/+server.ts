import type { RequestEvent } from '@sveltejs/kit';
import { json } from '@sveltejs/kit';
import { helixFetchStreamState } from '$lib/server/helix/utils';

export const GET = async (event: RequestEvent) => {
	// pull this out so that its properly formatted (when bothered)
	const API_URL = `https://api.piss.fan`;

	const uri = `${API_URL}/active-sockets`;
	const res = await fetch(uri, {
		method: 'GET'
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error while fetching active sockets:', res, body);
		return json(body ?? null, { status: res.status });
	}

	console.log(body);
	const streamQuery = await helixFetchStreamState(body.active_broadcasters);

	if (!streamQuery || streamQuery.status != 200) {
		console.error(
			'error while fetching stream state (see query output in console)'
		);
		return json(body, { status: streamQuery.status });
	}

	return json(streamQuery.body, { status: 200 });
};
