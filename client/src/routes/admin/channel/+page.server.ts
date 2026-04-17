import { invalidateCookie } from "$lib/server";
import {
	buildHeaders,
	buildHeadersAuthless,
	verifyToken,
} from "$lib/server/verify";
import { Rh } from "$lib/utils/route";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";
import { fail, type Actions } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

type BotConfig = {
	id: string;
	enabled: boolean;
	login: string;
	name: string;
	color: string;
	image: string;
};

async function getChannelConfigs(
	token: string,
	locals: App.Locals,
	id = "all"
): Promise<BotConfig[]> {
	const url = new URL(`${Rh.apiAdmin}/update/bot-config`);
	const logger = locals.logger.child({
		endpoint: url,
		channelId: id,
	});

	try {
		const headers = buildHeaders(true, token);
		url.searchParams.set("id", id);

		const res = await fetch(url, {
			method: "GET",
			headers,
		});

		if (!res.ok) {
			logger.warn({ response: res }, "failed to fetch bot configurations");
			return new Array();
		}

		const body = await res.json();
		return body.data as BotConfig[];
	} catch (err) {
		logger.error({ error: err }, "failure while fetching bot configurations");
		return new Array();
	}
}

async function getChannelHooks(
	token: string,
	locals: App.Locals,
	fetch: typeof globalThis.fetch
) {
	const endpoint = "/api/hooks";
	const logger = locals.logger.child({
		endpoint,
	});

	try {
		const headers = buildHeaders(false, token);
		const res = await fetch(endpoint, {
			method: "GET",
			headers,
		});

		if (!res.ok) {
			logger.warn({ response: res }, "failed to fetch hooks");
			return null;
		}

		const { data } = await res.json();
		return data;
	} catch (err) {
		logger.error({ error: err }, "failure while fetching hooks");
		return null;
	}
}

export const load: PageServerLoad = async ({ cookies, locals, fetch }) => {
	let token = await verifyToken(cookies, locals, fetch);
	if (token == null) {
		invalidateCookie(cookies);
		redirect(302, "/admin/login");
	}
	const configs = await getChannelConfigs(token, locals, "all");
	const hooks = await getChannelHooks(token, locals, fetch);
	return {
		configs,
		hooks,
	};
};

export const actions = {
	create: async ({ locals, request, fetch }) => {
		const logger = locals.logger.child({
			action: "channel-create",
		});

		try {
			const headers = buildHeadersAuthless(true);
			const formData = await request.formData();
			const channel = formData.get("channel") as string;

			if (channel === "" || channel.length < 4) {
				return fail(400, {
					error: "missing or invalid channel name",
				});
			}

			const res = await fetch("/api/channel", {
				method: "POST",
				headers,
				body: JSON.stringify({ user: channel }),
			});

			if (!res.ok) {
				logger.warn({ response: res }, "failed to process action");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			const body = await res.json();
			logger.info({ body }, "response");
			return { success: true, response: body.data };
		} catch (err) {
			logger.error({ error: err }, "failure while processing action");
			return fail(500, {
				error: err,
			});
		}
	},

	bot: async ({ locals, request, fetch }) => {
		const logger = locals.logger;

		try {
			const headers = buildHeadersAuthless(true);
			const formData = await request.formData();
			const id = formData.get("channel-id") as string;

			const res = await fetch("/api/channel/bot-state", {
				method: "PUT",
				headers,
				body: JSON.stringify({ id }),
			});

			if (!res.ok) {
				logger.warn("failed to process action");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			return { success: true };
		} catch (err) {
			logger.error(
				{ error: err },
				"failure while processing bot config action"
			);
			return fail(500, {
				error: err,
			});
		}
	},

	sync: async ({ locals, request, fetch }) => {
		const endpoint = "/api/channel/live-state";
		const logger = locals.logger.child({
			action: "channel-sync-state",
			endpoint,
		});

		try {
			const headers = buildHeadersAuthless(true);
			const formData = await request.formData();
			let id = formData.get("channel-id") as string;

			logger.info({ id });

			if (id === "") {
				id = "all";
			}

			const res = await fetch(endpoint, {
				method: "PUT",
				headers,
				body: JSON.stringify({ id }),
			});

			if (!res.ok) {
				logger.warn("failed to process action");
				return fail(res.status, {
					error: res.statusText,
				});
			}
			const body = await res.json();
			return { success: true, data: body.data };
		} catch (err) {
			logger.error({ error: err }, "failure while processing action");
			return fail(500, {
				error: err,
			});
		}
	},

	resethooks: async ({ locals, fetch }) => {
		const endpoint = `${Rh.apiAdmin}/helix/hooks`;
		const logger = locals.logger.child({
			action: "reset-hooks",
			endpoint,
		});

		try {
			const res = await fetch("/api/hooks", {
				method: "PUT",
			});

			if (!res.ok) {
				logger.warn({ response: res }, "failed to process action");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			return {
				success: true,
				data: "ok",
			};
		} catch (err) {
			logger.error({ error: err }, "failed while processing action");
			return { success: false, error: err };
		}
	},
} satisfies Actions;
