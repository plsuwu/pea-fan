import { logger } from "$lib/observability/server/logger.svelte";
import type { PageServerLoad } from "./$types";
import { Rh } from "$lib/utils/route";

const BOT_ENABLED_BROADCASTERS = `${Rh.apiBase}/channel/bot-enabled`;

export const load: PageServerLoad = async ({ fetch }) => {
	// const broadcasters = await fetchLiveBroadcasters(fetch);
	const botChannels = await fetchBotEnabledBroadcasters(fetch);

	return {
		// broadcasters,
		botChannels,
	};
};

async function fetchBotEnabledBroadcasters(fetch: typeof globalThis.fetch) {
	const res = await fetch(BOT_ENABLED_BROADCASTERS, {
		method: "GET",
	});

	if (res.status != 200) {
		logger.error(
			{ response: res, url: BOT_ENABLED_BROADCASTERS },
			"failed to get bot enabled list"
		);
		return null;
	}

	const body = await res.json();
	return body;
}
