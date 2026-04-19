import { apiBucket } from "$lib/server/rate-limiter/token-bucket";
import { env } from "$env/dynamic/private";
import { json, type RequestHandler } from "@sveltejs/kit";
import { buildHeadersAuthless } from "$lib/server/verify";

let WEBHOOK_URL = env.DISCORD_WEBHOOK_URL;

export const POST: RequestHandler = async ({ request, locals, url, fetch }) => {
	const logger = locals.logger.child({
		route: url.href,
	});

	try {
		if (!apiBucket.consume(locals.client.cfconnecting, 1)) {
			logger.warn("rejecting rate limited client");
			return json(
				{ status: 429, error: "rate limit exceeded" },
				{ status: 429 }
			);
		}

		const input = await request.json();
		const headers = buildHeadersAuthless(true);

		const body = JSON.stringify({
			content: JSON.stringify({
				"cf-client": locals.client.cfconnecting,
				...input,
			}),
		});

		const res = await fetch(WEBHOOK_URL, {
			method: "POST",
			headers,
			body,
		});

		if (!res.ok) {
			logger.warn({ response: res }, "failed to POST to webhook");
			return json({ status: res.status, error: res.statusText });
		}

		return json({ status: 204, data: "ok" });
	} catch (err) {
		logger.error({ error: err }, "failure during webhook post");
		return json({ status: 500, err: err });
	}
};
