import { buildHeaders } from "$lib/server/verify";
import type { RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { json } from "@sveltejs/kit";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const RESET_HOOKS_ENDPOINT = `${Rh.apiBase}/helix/reset-hooks`;

export const GET: RequestHandler = async ({ locals, cookies }) => {
	let token = cookies.get(ADMIN_SESSION_TOKEN);
	if (!token) {
		return json({ valid: false });
	}

	const headers = buildHeaders(false, token);
	const res = await fetch(RESET_HOOKS_ENDPOINT, {
		method: "GET",
		headers,
	});

	locals.logger.debug({ response: res }, "api validation response");
	if (res.status === 200 && res.statusText === "OK") {
		return json({ success: false });
	}

	return json({ valid: false });
};

