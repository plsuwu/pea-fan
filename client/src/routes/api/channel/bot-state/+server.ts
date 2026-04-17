import { env } from "$env/dynamic/private";
import { buildHeaders } from "$lib/server/verify";
import { Rh } from "$lib/utils/route";
import { json, type RequestHandler } from "@sveltejs/kit";

const CONFIG_ENDPOINT = `${Rh.apiAdmin}/update/bot-config`;
const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

export const PUT: RequestHandler = async ({
	cookies,
	locals,
	request,
	fetch,
}) => {
	const logger = locals.logger.child({
		method: request.method,
		endpoint: CONFIG_ENDPOINT,
	});

	try {
		const token = cookies.get(ADMIN_SESSION_TOKEN);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "unauthorized" });
		}

		const { id } = await request.json();
		logger.info({ id }, "");
		const headers = buildHeaders(true, token);

		const res = await fetch(CONFIG_ENDPOINT, {
			method: "PUT",
			headers,
			body: JSON.stringify({ id }),
		});

		if (!res.ok) {
			logger.warn({ response: res }, "failed to run bot config update");
			return json({ status: res.status, data: res.statusText });
		}

		return json({ status: res.status, data: "ok" });
	} catch (err) {
		logger.error({ error: err }, "failure during bot config update");
		return json({ status: 500, data: err });
	}
};
