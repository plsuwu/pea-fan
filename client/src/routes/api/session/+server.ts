import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
// import { logger } from "$lib/observability/server/logger.svelte";
import { buildHeaders } from "$lib/server/verify";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

export const GET: RequestHandler = async ({ cookies, locals, fetch }) => {
	const sessionUri = `${Rh.apiAdmin}/session`;

	const logger = locals.logger.child({
		sessionUri,
	});

	// logger.info("starting serverside token validation");

	let token = cookies.get(ADMIN_SESSION_TOKEN);
	if (token == null) {
		logger.warn("request is missing admin token");
		return json({ status: 400, data: "invalid session token" });
	}

	try {
		const headers = buildHeaders(true, token);
		const res = await fetch(sessionUri, {
			method: "GET",
			headers,
		});

		if (!res.ok) {
			logger.warn(
				{ response: res, token: token },
				"verification for token failed (received non-200 response)"
			);
			return json({ status: 400, data: "invalid session token" });
		}

		const body = await res.json();
		if (body.data !== "ok" || body.status != 200) {
			logger.warn(
				{ response: res, body, token: token },
				"verification for token failed (JSON body mismatch)"
			);
			return json({ status: 400, data: "invalid session token" });
		}

		// logger.trace({ body }, "valid token");
		return json({ status: 200, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process validation");
		return json({ status: 500, data: err });
	}
};
