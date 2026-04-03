import type { Cookies } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { logger } from "$lib/observability/server/logger.svelte";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
export async function verifyToken(
	cookies: Cookies,
	request: Request,
	getClientAddress: () => string,
	fetch: typeof globalThis.fetch
): Promise<string | null> {
	const token = cookies.get(ADMIN_SESSION_TOKEN);
	if (!token) {
		logger.warn(
			{ ...getClientInfo(request, getClientAddress) },
			"unauthorized (no session cookie)"
		);
		return null;
	}

	const res = await fetch("/api/verify-session", {
		method: "POST",
		headers: buildHeaders(true, token),
		body: JSON.stringify({ token }),
	});

	const { valid } = await res.json();
	logger.info({ valid }, "initial session validation response");

	if (token && valid !== true) {
		logger.warn(
			{ ...getClientInfo(request, getClientAddress) },
			"unauthorized"
		);

		return null;
	}

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

export function getClientInfo(
	request: Request,
	getClientAddress: () => string
) {
	const userAgent = request.headers.get("user-agent") ?? "[NO_USER_AGENT]";
	const host = request.headers.get("host") ?? "[NO_HOST]";
	const cookie = request.headers.get("cookie") ?? "[NO_COOKIE]";
	return {
		client: {
			addr: getClientAddress(),
			url: request.url,
			headers: {
				user_agent: userAgent,
				host,
				cookies: cookie,
			},
		},
	};
}
