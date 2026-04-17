import type { PageServerLoad } from "./$types";
import type { Actions } from "@sveltejs/kit";
import { fail, redirect } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie } from "$lib/server";
import { Rh } from "$lib/utils/route";
import { buildHeaders, verifyToken } from "$lib/server/verify";

// TODO:
// -------------------------------------------------------------------
// - endpoint needs rate limit hook,
// - perhaps don't need to perform token verification on all actions?

async function getChannelConfigs(token: string, id = "all") {
	const headers = buildHeaders(true, token);
	const url = new URL(`${Rh.apiAdmin}/update/bot-config`);

	url.searchParams.set("id", id);
	const res = await fetch(url, {
		method: "GET",
		headers,
	});

	if (!res.ok) {
		logger.error({ response: res }, "failed to fetch bot configurations");
		return null;
	}

	const body = await res.json();
	return body.data;
}

export const load: PageServerLoad = async ({ cookies, url, fetch, locals }) => {
	if (locals.channel) {
		redirect(302, `${Rh.proto}://${Rh.deriveBase(url.host)}/admin`);
	}

	let token = await verifyToken(cookies, locals, fetch);

	if (token == null) {
		invalidateCookie(cookies);
		redirect(302, "/admin/login");
	}

	const channels = await getChannelConfigs(token);
	return {
		channels,
	};
};

const isDigits = (str: string) => /^\d+$/.test(str);

function getHelixQueryType(user: string): string {
	if (isDigits(user) && user.length >= 8 && user.length <= 10) {
		return "by-id";
	}

	return "by-login";
}

export const actions = {
	helix: async ({ request, fetch, locals, cookies }) => {
		const logger = locals.logger.child({
			action: "search-helix",
		});

		const token = await verifyToken(cookies, locals, fetch);
		if (token == null) {
			logger.warn("missing token");
			return fail(400, {
				error: "missing token",
			});
		}

		try {
			const headers = buildHeaders(false, token);
			const formData = await request.formData();
			const user = formData.get("user") as string;

			if (user === "" || user.length < 3) {
				logger.warn({ user: user }, "missing or invalid user identifier");
				return fail(400, {
					error: "missing or invalid user identifier",
				});
			}

			const endpoint = `${Rh.apiAdmin}/helix/${getHelixQueryType(user)}/${user}`;
			logger.setBindings({
				endpoint,
				user,
			});

			const res = await fetch(endpoint, {
				method: "GET",
				headers,
			});

			if (!res.ok) {
				logger.error({ response: res }, "helix query failed");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			const body = await res.json();
			return { status: 200, data: body.data };
		} catch (err) {
			logger.error({ error: err }, "failure during action");
			return fail(500, {
				error: err,
			});
		}
	},

	database: async ({ request, fetch, locals }) => {
		const logger = locals.logger.child({
			action: "search-database",
		});

		try {
			const formData = await request.formData();
			const user = formData.get("user") as string;

			const endpoint = `${Rh.apiv1}/search/${user}`;
			const res = await fetch(endpoint, {
				method: "GET",
			});

			if (!res.ok) {
				logger.error({ response: res }, "chatter query fail");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			const body = await res.json();
			logger.info({ response: res, body }, "query ok");

			return { status: 200, data: body.data?.[0] };
		} catch (err) {
			logger.error({ error: err }, "failure during action");
			return fail(500, {
				error: err,
			});
		}
	},

	// getActiveHooks: async ({ fetch }) => {
	// 	const res = await fetch("/api/hooks", {
	// 		method: "GET",
	// 	});
	//
	// 	if (res.status !== 200) {
	// 		return { success: false, status: res.status };
	// 	}
	//
	// 	logger.info({ response: res }, "hooks retrieved ok");
	// 	const { hooks } = await res.json();
	// 	return {
	// 		success: true,
	// 		from: "getActiveHooks",
	// 		results: hooks,
	// 	};
	// },
	//
	// deleteHooks: async ({ fetch }) => {
	// 	const res = await fetch("/api/hooks", {
	// 		method: "DELETE",
	// 	});
	//
	// 	if (res.status !== 200) {
	// 		return { success: false, status: res.status };
	// 	}
	//
	// 	logger.info({ response: res }, "hooks deleted ok");
	// 	return {
	// 		success: true,
	// 	};
	// },
	//
	// resetHooks: async ({ fetch }) => {
	// 	const res = await fetch("/api/hooks", {
	// 		method: "PUT",
	// 	});
	//
	// 	if (res.status !== 200) {
	// 		return { success: false, status: res.status };
	// 	}
	//
	// 	logger.info({ response: res }, "hooks reset ok");
	// 	return {
	// 		success: true,
	// 	};
	// },
} satisfies Actions;

// const UPDATE_API_ROUTE = `${API_BASE}/update`;

async function runUpdate(
	keytype: "channel" | "chatter",
	data: { current: string; historic: string[] },
	headers: Headers,
	fetch: typeof globalThis.fetch
) {
	const updateEndpoint = `${Rh.apiAdmin}/update/${keytype}`;
	const res = await fetch(updateEndpoint, {
		method: "PUT",
		headers,
		body: JSON.stringify(data),
	});

	if (!res.ok) {
		logger.error({ response: res }, "failed to complete action");
	}

	const body = await res.json();
	logger.info({ body: body.status }, "server response status");

	return {
		success: res.ok,
		status: res.status,
	};
}

// async function runMerge(fetch: typeof globalThis.fetch, headers: Headers) {
// 	const mergeEndpoint = `${UPDATE_API_ROUTE}/migrate`;
// 	const res = await fetch(mergeEndpoint, {
// 		method: "GET",
// 		keepalive: true,
// 		headers,
// 	});
//
// 	if (!res.ok) {
// 		logger.error({ response: res }, "[ACTION] update failed");
// 		return { success: false, status: res.status, body: "" };
// 	}
//
// 	const body = await res.json();
// 	logger.info({ body }, "RX from server");
//
// 	return {
// 		success: true,
// 		status: res.status,
// 		body,
// 	};
// }
