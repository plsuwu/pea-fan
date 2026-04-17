import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { logger } from "$lib/observability/server/logger.svelte";
import { apiRateLimiter } from "$lib/server/rate-limit.svelte";
import { buildHeadersAuthless } from "$lib/server/verify";

let WEBHOOK_URL = env.DISCORD_WEBHOOK_URL;
const MIN_TWITCH_USERNAME_LEN = 4;
const MAX_TWITCH_USERNAME_LEN = 25;

export const POST: RequestHandler = async (event: RequestEvent) => {
	const data = await event.request.json();
	const { current, previous, score, comment, requestingClient } = data;

	// if (
	// 	!current ||
	// 	!score ||
	// 	current.length < MIN_TWITCH_USERNAME_LEN ||
	// 	current.length > MAX_TWITCH_USERNAME_LEN ||
	// 	(previous && previous.length < MIN_TWITCH_USERNAME_LEN) ||
	// 	previous.length > MAX_TWITCH_USERNAME_LEN
	// ) {
	//        // handle invalid entry
	// }

	const now = new Date();

	const body = JSON.stringify({
		content: JSON.stringify({
			current,
			previous,
			score,
			comment,
			requestingClient,
			now,
		}),
	});

	const res = await fetch(WEBHOOK_URL, {
		method: "POST",
		body,
		headers: buildHeadersAuthless(true),
	});

	return json({ status: null });
};
