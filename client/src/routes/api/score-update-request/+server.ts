import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { logger } from "$lib/observability/server/logger.svelte";
import { apiRateLimiter } from "$lib/server/rate-limit.svelte";

let WEBHOOK_URL = env.DISCORD_WEBHOOK_URL;
const MIN_TWITCH_USERNAME_LEN = 4;
const MAX_TWITCH_USERNAME_LEN = 25;

export const POST: RequestHandler = async (event: RequestEvent) => {
	const childLogger = event.locals.logger.child({
		client: event.locals.client,
	});

	if (!apiRateLimiter.consume(event.locals.client.cfconnecting, 3)) {
		childLogger.warn("[API::SCORE_UPDATE]: FAIL_RATE_LIMITED_CLIENT");
		return json(
			{ success: false, reason: "rate limit exceeded" },
			{ status: 429 }
		);
	}

	const req = event.request;
	const data = await req.json();
	const { current, previous, score, comment, requestingClient } = data;

	childLogger.setBindings({
		current,
		previous,
		score,
		comment,
		requestingClient,
	});

	if (
		!current ||
		!score ||
		current.length < MIN_TWITCH_USERNAME_LEN ||
		current.length > MAX_TWITCH_USERNAME_LEN ||
		(previous && previous.length < MIN_TWITCH_USERNAME_LEN) ||
		previous.length > MAX_TWITCH_USERNAME_LEN
	) {
		childLogger.error("[API::SCORE_UPDATE]: FAIL_INVALID_POST_BODY");
		return json(
			{ success: false, reason: "invalid POST body" },
			{ status: 400 }
		);
	}

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
		headers: {
			"content-type": "application/json",
		},
	});

	if (res.status != 204) {
		childLogger.error(
			{ response: res },
			"[API::SCORE_UPDATE]: FAIL_RESPONSE_NON_204"
		);
	}

	return json({ success: true });
};
