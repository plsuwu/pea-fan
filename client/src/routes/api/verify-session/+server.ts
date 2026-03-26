import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { logger } from "$lib/observability/server/logger.svelte";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const TOKEN_ENDPOINT = new URL(
	`${Rh.apiBase}/auth/validate-session`
);

const buildHeaders = (token: string): Headers => {
	const headers = new Headers();
	headers.set("content-type", "application/json");
	headers.set("authorization", token);

	return headers;
};

export const GET: RequestHandler = async (event: RequestEvent) => {
	let token = event.cookies.get(ADMIN_SESSION_TOKEN);
	if (!token) {
		return json({ valid: false });
	}

	const headers = buildHeaders(token);
	const res = await fetch(TOKEN_ENDPOINT, {
		method: "GET",
		headers,
	});

	logger.debug({ response: res }, "api validation response");
	if (res.status === 200 && res.statusText === "OK") {
		return json({ valid: true });
	}

	return json({ valid: false });
};

// we dont actually need to do anything with data, but idk if this is 
// problematic so ideally get rid of this in future!
export const POST: RequestHandler = async (event: RequestEvent) => {
	return await event.fetch("/api/verify-session");
};
