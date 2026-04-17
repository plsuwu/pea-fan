import { buildHeaders, verifyToken } from "$lib/server/verify";
import { Rh } from "$lib/utils/route";
import { json, type RequestHandler } from "@sveltejs/kit";
import { type Logger } from "pino";

const API_ENDPOINT = `${Rh.apiAdmin}/update/live`;

export const PUT: RequestHandler = async ({
	cookies,
	locals,
	fetch,
	request,
}) => {
	const logger: Logger = locals.logger.child({
		method: request.method,
		endpoint: API_ENDPOINT,
	});

	try {
		const token = await verifyToken(cookies, locals, fetch);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const headers = buildHeaders(true, token);
		const { id } = await request.json();

		const endpoint = new URL(API_ENDPOINT);
		logger.setBindings({
			channelId: id,
			endpoint: endpoint,
		});

		const res = await fetch(endpoint, {
			method: "PUT",
			headers,
			body: JSON.stringify({ id }),
		});

		if (!res.ok) {
			logger.warn("failed to update live state for channel");
			return json({ status: res.status, data: "failed to update state" });
		}

		// logger.debug("successfully updated live state");
		return json({ status: res.status, data: "ok" });
	} catch (err) {
		logger.error({ error: err }, "error while syncing live state");
		return json({ status: 500, data: "failed to update state" });
	}
};
