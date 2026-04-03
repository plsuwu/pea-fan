import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";
import { buildHeaders } from "$lib/server/verify";

const TOKEN_ENDPOINT = new URL(`${Rh.apiBase}/auth/totp-session`);

// const buildHeaders = (): Headers => {
// 	const headers = new Headers();
// 	headers.set("content-type", "application/json");
//
// 	return headers;
// };

export const POST: RequestHandler = async (event: RequestEvent) => {
	const { request } = event;

	try {
		const { token } = await request.json();
		if (!token) {
			return json({ valid: false, session: null });
		}

		let response = await fetch(TOKEN_ENDPOINT, {
			method: "POST",
			headers: buildHeaders(true, token),
			body: JSON.stringify({ token }),
		});

		logger.info({ response }, "API RESPONSE");

		if (response.status !== 200) {
			return json({ valid: false, session: null });
		}

		let body = await response.json();
		logger.info({ valid: body.valid }, "validation result");
		return json({ valid: body.valid, session: body.session });
	} catch (err) {
		logger.error({ error: err }, "error during TOTP validation");
		return json({ valid: false, session: null });
	}
};
