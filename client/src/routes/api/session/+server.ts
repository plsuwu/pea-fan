import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
// import { logger } from "$lib/observability/server/logger.svelte";
import { buildHeaders } from "$lib/server/verify";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

export const GET: RequestHandler = async ({ cookies, locals }) => {
    const sessionUri = `${Rh.apiAdmin}/session`;

	const logger = locals.logger.child({
        sessionUri,
    });

	logger.info("starting serverside token validation");

	let token = cookies.get(ADMIN_SESSION_TOKEN);
	if (token == null) {
		logger.warn("missing admin token");
		return json({ status: 400, data: "invalid session token" });
	}

	try {
		const headers = buildHeaders(true, token);
		const res = await fetch(sessionUri, {
			method: "GET",
			headers,
		});

		const body = await res.json();
		if (res.status !== 200) {
			logger.warn({ token: token }, "verification for token failed");
			return json({ status: 400, data: "invalid session token" });
		}

		logger.debug("valid token");
		return json({ status: 200, data: body });
	} catch (err) {
		logger.error({ error: err }, "unable to process validation");
		return json({ status: 500, data: err });
	}
};
