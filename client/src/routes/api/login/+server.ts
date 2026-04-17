import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { Rh } from "$lib/utils/route";
import { buildHeaders } from "$lib/server/verify";

const TOKEN_ENDPOINT = new URL(`${Rh.apiAdmin}/auth/new-session`);

export const POST: RequestHandler = async ({ locals, request, fetch }) => {
	const { logger } = locals;
	logger.debug("starting serverside login handler");

	const { token } = await request.json();
	if (token == null) {
		logger.warn("missing token");
		return json({ status: 400, data: "invalid token" });
	}

	try {
		let res = await fetch(TOKEN_ENDPOINT, {
			method: "POST",
			headers: buildHeaders(true, token),
			body: JSON.stringify({ token }),
		});

		const body = await res.json();
		if (res.ok != true) {
			logger.warn({ token }, "token did not match");
			return json({ status: 401, data: "could not validate this token" });
		}

		logger.debug({ body: body }, "successful login");
		return json({ status: 200, data: body });
	} catch (err) {
		logger.error({ error: err }, "unable to process login request");
		return json({ status: 500, data: err });
	}
};
