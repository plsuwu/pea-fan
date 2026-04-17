import type { Cookies } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
export async function verifyToken(
	cookies: Cookies,
	locals: App.Locals,
	fetch: typeof globalThis.fetch
): Promise<string | null> {
	const token = cookies.get(ADMIN_SESSION_TOKEN);
	const logger = locals.logger;

	if (token == null) {
		logger.warn("unauthorized: missing token");
		return null;
	}

	const res = await fetch("/api/session", {
		method: "GET",
		headers: buildHeaders(true, token),
	});

	const body = await res.json();
	const { status, data } = body;

	if (token && status !== 200 && data !== "ok") {
		logger.warn("unauthorized: invalid token");

		return null;
	}

	// logger.info("validation passed for client");
	return token;
}

export function buildHeadersAuthless(isJSON: boolean) {
	const headers = new Headers();
	if (isJSON) {
		headers.set("content-type", "application/json");
	}

	return headers;
}

export function buildHeaders(isJSON: boolean, token: string) {
	const headers = new Headers();
	headers.set("authorization", token);

	if (isJSON) {
		headers.set("content-type", "application/json");
	}

	return headers;
}

export function getClientInfo(request: Request, locals: App.Locals) {
	const client = locals.client.cfconnecting;
	const userAgent = request.headers.get("user-agent") ?? "[NO_USER_AGENT]";
	const host = request.headers.get("host") ?? "[NO_HOST]";
	const cookie = request.headers.get("cookie") ?? "[NO_COOKIE]";
	return {
		client: {
			cfconnecting: client,
			url: request.url,
			headers: {
				user_agent: userAgent,
				host,
				cookies: cookie,
			},
		},
	};
}
