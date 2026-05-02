import type { PageServerLoad } from "./$types";
import type { Actions } from "@sveltejs/kit";
import { fail, redirect } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie } from "$lib/server";
import { routeManager } from "$lib/utils/route";
import { buildHeaders, verifyToken } from "$lib/server/verify";

export const load: PageServerLoad = async ({ cookies, url, fetch, locals }) => {
	if (locals.channel) {
		// redirect to base host URL if the client is on a tenant subdomain
		const adminLogin = `${routeManager.getUntenantedURL(url.host)}/admin`;
		redirect(302, adminLogin);
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

			// NOTE that it is technically possible for users to have 1-3 character
			// names, though these are usually antiquated or twitch staff.
			// e.g.:
			//  - https://www.twitch.tv/x
			//  - https://www.twitch.tv/fig (among others)
			//  - ...
			if (user === "") {
				logger.warn({ user: user }, "missing or invalid user identifier");
				return fail(400, {
					error: "missing or invalid user identifier",
				});
			}

			const endpoint = routeManager.internApiUrl(
				"_admin",
				`helix/${getHelixQueryType(user)}/${user}`
			);

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

			const endpoint = routeManager.internApiUrl("search", user);
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
} satisfies Actions;

const isDigits = (str: string) => /^\d+$/.test(str);
function getHelixQueryType(user: string): string {
	if (isDigits(user) && user.length >= 7 && user.length <= 11) {
		return "by-id";
	}

	return "by-login";
}

async function getChannelConfigs(token: string, id = "all") {
	const headers = buildHeaders(true, token);
	const url = new URL(routeManager.internApiUrl("_admin", "update/bot-config"));

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
