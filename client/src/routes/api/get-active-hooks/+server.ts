import { buildHeaders } from "$lib/server/verify";
import type { RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { json } from "@sveltejs/kit";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const ACTIVE_HOOKS_ENDPOINT = `${Rh.apiBase}/helix/get-hooks`;

export const GET: RequestHandler = async ({ locals, cookies }) => {
	let token = cookies.get(ADMIN_SESSION_TOKEN);
	if (!token) {
		return json({ valid: false });
	}

	const headers = buildHeaders(false, token);
	const res = await fetch(ACTIVE_HOOKS_ENDPOINT, {
		method: "GET",
		headers,
	});

	locals.logger.debug({ response: res }, "[API]: hooks GET resp");
	if (res.status !== 200) {
		locals.logger.error({ response: res }, "failed to get hooks from API");
		return json({ success: false });
	}

    const body = await res.json();
    console.log(body)

	return json({ success: true, hooks: body });
};
