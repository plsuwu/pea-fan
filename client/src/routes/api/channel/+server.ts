import { env } from "$env/dynamic/private";
import { buildHeaders, verifyToken } from "$lib/server/verify";
import { Rh } from "$lib/utils/route";
import { json, type RequestHandler } from "@sveltejs/kit";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const CHANNELS_ENDPOINT = `${Rh.apiAdmin}/update/channels`;

// export const PUT: RequestHandler = async ({ locals, cookies, request }) => {};

export const POST: RequestHandler = async ({
	locals,
	cookies,
	request,
	fetch,
}) => {
	const logger = locals.logger.child({
		method: request.method,
		endpoint: CHANNELS_ENDPOINT,
	});

	try {
		let token = await verifyToken(cookies, locals, fetch);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const headers = buildHeaders(true, token);
		const body = await request.json();

		logger.info({ body }, "post body");

		const res = await fetch(CHANNELS_ENDPOINT, {
			method: "POST",
			headers,
			body: JSON.stringify(body),
		});

		if (!res.ok) {
			logger.warn({ response: res }, "failed to add new channel");
			return json({ status: res.status, data: "failed to insert" });
		}

		const resBody = await res.json();
		return json({ status: 200, data: resBody.data });
	} catch (err) {
		logger.error("failure during channel POST");
		return json({ status: 500, data: err });
	}
};
