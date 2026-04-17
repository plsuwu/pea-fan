import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { Rh } from "$lib/utils/route";
import { buildHeaders } from "$lib/server/verify";

const TOKEN_ENDPOINT = new URL(`${Rh.apiv1}/auth/new-session`);

export const POST: RequestHandler = async ({ locals, request, fetch }) => {
	const logger = locals.logger.child({
		url: TOKEN_ENDPOINT,
	});

	logger.debug("starting serverside login handler");

	try {
		// let token = null;

		const { token } = await request.json();

		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		logger.info({ token: token }, "fetching session using token");

		let res = await fetch(TOKEN_ENDPOINT, {
			method: "POST",
			headers: buildHeaders(true, token),
			body: JSON.stringify({ token }),
		});

		if (res.ok != true) {
			logger.warn({ token }, "token did not match");
			return json({ status: 400, data: "invalid token" });
		}

		const body = await res.json();

		if (body.data.is_valid !== true) {
			logger.warn({ body }, "unsuccessful login attempted");
			return json({ status: 400, data: "invalid token" });
		}

		logger.debug({ body: body.data }, "successful login");
		return json({ status: 200, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "error while processing login request");
		return json({ status: 500, data: err });
	}
};
