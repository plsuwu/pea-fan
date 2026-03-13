import { ADMIN_SESSION_TOKEN } from "$env/static/private";
import type { PageServerLoad } from "./$types";
import type { Actions, Cookies } from "@sveltejs/kit";
import { redirect } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie } from "$lib/server";
import { Rh } from "$lib/utils/route";
import { channelCache } from "$lib/observability/server/cache.svelte";

// TODO:
// -------------------------------------------------------------------
// - endpoint needs rate limit hook,
// - HMAC-based message signing/verification,
// - also probably perform verification (or part of the verification)
//    in a server hook instead of here.

const API_BASE = `${Rh.proto}://${Rh.api}`;
const FETCH_CONFIGS = `${API_BASE}/channel/reply-configs`;
const UPDATE_CONFIGS = `${API_BASE}/update/reply-configs`;

function getClientInfo(request: Request, getClientAddress: () => string) {
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

async function getChannelConfigs(token: string, id = "all") {
	const headers = buildHeaders(true, token);
	const url = new URL(FETCH_CONFIGS);

	url.searchParams.set("id", id);
	const res = await fetch(url, {
		method: "GET",
		headers,
	});

	if (res.status !== 200) {
		logger.error({ response: res }, "failed to fetching channel configs");
		return null;
	}

	const body = await res.json();
	return [...body];
}

export const load: PageServerLoad = async ({
	fetch,
	cookies,
	request,
	getClientAddress,
}) => {
	let token = await verifyToken(cookies, request, getClientAddress, fetch);
	if (!token) {
		invalidateCookie(cookies);
		redirect(302, "/admin/login");
	}

	const channels = await getChannelConfigs(token);
	return {
		channels,
	};
};

async function verifyToken(
	cookies: Cookies,
	request: Request,
	getClientAddress: () => string,
	fetch: typeof globalThis.fetch
): Promise<string | null> {
	const token = cookies.get(ADMIN_SESSION_TOKEN);
	if (!token) {
		logger.warn("no session cookie");
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

function buildHeaders(isJSON: boolean, token: string) {
	const headers = new Headers();
	headers.set("authorization", token);

	if (isJSON) {
		headers.set("content-type", "application/json");
	}

	return headers;
}

export const actions = {
	update: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);
		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const formData = await request.formData();
		const type = formData.get("type") as string as "channel" | "chatter";

		const current = formData.get("current") as string;
		const historic = formData.get("historic") as string;

		if (!type || !current || !historic) {
			return {
				success: false,
				message: "missing one of 'type'/'current'/'historic'",
			};
		}

		const headers = buildHeaders(true, token);
		const data = { current, historic: JSON.parse(historic) };

		const { success, status, body } = await runUpdate(
			type,
			data,
			fetch,
			headers
		);

		logger.debug({ success, status, body }, "update action complete");
		return { success, status, body };
	},

	merge: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);

		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const headers = buildHeaders(false, token);
		const { success, status, body } = await runMerge(fetch, headers);

		logger.debug({ success, status, body }, "merge action complete");
		return { success, status, body };
	},

	toggleReply: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);
		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const headers = buildHeaders(true, token);
		const formData = await request.formData();
		const id = formData.get("id") as string;

		const body = { id };

		console.log(UPDATE_CONFIGS, id, body);

		const res = await fetch(UPDATE_CONFIGS, {
			method: "POST",
			headers,
			body: JSON.stringify({ id }),
		});

		if (res.status !== 200) {
			logger.error({ response: res }, "config update failed");
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "config update successful");
		return { success: true };
	},
} satisfies Actions;

const UPDATE_API_ROUTE = `${Rh.proto}://${Rh.api}/update`;

async function runUpdate(
	keytype: "channel" | "chatter",
	data: { current: string; historic: string[] },
	fetch: typeof globalThis.fetch,
	headers: Headers
) {
	const updateEndpoint = `${UPDATE_API_ROUTE}/${keytype}`;
	const res = await fetch(updateEndpoint, {
		method: "POST",
		headers,
		body: JSON.stringify(data),
	});

	if (!res.ok) {
		logger.error({ response: res }, "update failed");
		return { success: false, body: "", status: res.status };
	}

	const body = await res.json();
	logger.info({ body }, "server response");

	return {
		success: body === "OK",
		status: res.status,
		body,
	};
}

async function runMerge(fetch: typeof globalThis.fetch, headers: Headers) {
	const mergeEndpoint = `${UPDATE_API_ROUTE}/migrate`;
	const res = await fetch(mergeEndpoint, {
		method: "GET",
		keepalive: true,
		headers,
	});

	if (!res.ok) {
		logger.error({ response: res }, "[ACTION] update failed");
		return { success: false, status: res.status, body: "" };
	}

	const body = await res.json();
	logger.info({ body }, "RX from server");

	return {
		success: true,
		status: res.status,
		body,
	};
}
