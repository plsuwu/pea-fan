import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { logger } from "$lib/observability/server/logger.svelte";

let WEBHOOK_URL = env.DISCORD_WEBHOOK_URL;

export const POST: RequestHandler = async (event: RequestEvent) => {
	const req = event.request;
	const data = await req.json();

	const { current, previous, score } = data;
	if (!current || !previous || !score) {
		logger.warn("discarding garbage request");
		return json({ success: true });
	}

	const body = JSON.stringify({
		content: JSON.stringify({
			current,
			previous,
			score,
		}),
	});

	console.log(body);

	const res = await fetch(WEBHOOK_URL, {
		method: "POST",
		body,
		headers: {
			"content-type": "application/json",
		},
	});

	console.log("webhook res:");
	console.log(res);

	return json({ success: true });
};
