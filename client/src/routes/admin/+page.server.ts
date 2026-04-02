import type { PageServerLoad } from "./$types";
import type { Actions } from "@sveltejs/kit";
import { redirect } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie } from "$lib/server";
import { Rh } from "$lib/utils/route";
import { buildHeaders, verifyToken } from "$lib/server/verify";
import { adminRateLimiter } from "$lib/server/rate-limit.svelte";

// TODO:
// -------------------------------------------------------------------
// - endpoint needs rate limit hook,
// - HMAC-based message signing/verification,
// - also probably perform (some of) the verification in a server hook instead of here.

// const API_BASE = `${Rh.apiProto}://${Rh.api}`;
const API_BASE = `${Rh.apiBase}`;
const FETCH_CONFIGS = `${API_BASE}/channel/reply-configs`;
const UPDATE_CONFIGS = `${API_BASE}/update/reply-configs`;
const CLEAR_CHATTER_SCORE = `${API_BASE}/update/clear-scores/chatter`;
const HELIX_PROXY_SEARCH = `${API_BASE}/helix/by-login`;

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
	locals,
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

	refreshChannel: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);

		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const headers = buildHeaders(false, token);
		const res = await fetch(`${API_BASE}/update/channel?login=all`, {
			method: "GET",
			headers,
		});

		if (res.status !== 200) {
			logger.error({ response: res }, "full channel update failed");
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "full channel update successful");
		return { success: true };
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

	clearScore: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);
		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const headers = buildHeaders(false, token);
		const formData = await request.formData();
		const id = formData.get("id") as string;

		const res = await fetch(`${CLEAR_CHATTER_SCORE}/${id}`, {
			method: "GET",
			headers,
		});

		if (res.status !== 200) {
			logger.error({ response: res, id: id }, "failed to clear chatter score");
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "successfully cleared chatter score");
		return { success: true };
	},

	searchHelix: async ({ cookies, request, fetch, getClientAddress }) => {
		const token = await verifyToken(cookies, request, getClientAddress, fetch);
		if (!token) {
			invalidateCookie(cookies);
			return { success: false };
		}

		const headers = buildHeaders(false, token);
		const formData = await request.formData();
		const login = formData.get("login") as string;

		const res = await fetch(`${HELIX_PROXY_SEARCH}/${login}`, {
			method: "GET",
			headers,
		});

		if (res.status !== 200) {
			logger.error({ response: res, login: login }, "helix query failed");
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "query ok");
		return { success: true, from: "searchHelix", results: await res.json() };
	},

	searchChatter: async ({ request, fetch }) => {
		const formData = await request.formData();
		const query = formData.get("login") as string;

		const url = new URL(`${API_BASE}/search/by-login`);
		url.searchParams.set("login", query);

		const res = await fetch(url, {
			method: "GET",
		});

		if (res.status !== 200) {
			logger.error({ response: res, query: query }, "chatter query failed");
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "query ok");
		return {
			success: true,
			from: "searchChatter",
			results: await res.json(),
		};
	},

	getActiveHooks: async ({ fetch }) => {
		const res = await fetch("/api/get-active-hooks", {
			method: "GET",
		});

		if (res.status !== 200) {
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "hooks retrieved ok");
        const { hooks } = await res.json();
		return {
			success: true,
			from: "getActiveHooks",
			results: hooks,
		};
	},

	deleteHooks: async ({ fetch }) => {
		const res = await fetch("/api/delete-hooks", {
			method: "GET",
		});

		if (res.status !== 200) {
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "hooks deleted ok");
		return {
			success: true,
		};
	},

	resetHooks: async ({ fetch }) => {
		const res = await fetch("/api/reset-hooks", {
			method: "GET",
		});

		if (res.status !== 200) {
			return { success: false, status: res.status };
		}

		logger.info({ response: res }, "hooks reset ok");
		return {
			success: true,
		};
	},
} satisfies Actions;

const UPDATE_API_ROUTE = `${API_BASE}/update`;

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
