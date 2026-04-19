import { fail, redirect, type Actions } from "@sveltejs/kit";
import { apiBucket } from "$lib/server/rate-limiter/token-bucket";
import { invalidateCookie, setCookie } from "$lib/server";
import type { PageServerLoad } from "../$types";
import { env } from "$env/dynamic/private";
import { buildHeadersAuthless } from "$lib/server/verify";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

export const load: PageServerLoad = async ({ cookies, locals, url }) => {
	const hasToken = cookies.get(ADMIN_SESSION_TOKEN);
	if (hasToken) {
		locals.logger.warn(
			{ token: hasToken },
			"session token present, redirecting instead"
		);

		redirect(302, "/admin");
	}

	const redirectReason = url.searchParams.get("err") || null;
	return {
		redirectReason,
	};
};

export const actions = {
	default: async ({ cookies, request, fetch, locals }) => {
		const logger = locals.logger;

		if (!apiBucket.consume(locals.client.cfconnecting, 1)) {
			logger.warn("client rate limited on /admin/login");
			return fail(429, { reason: "rate limit exceeded" });
		}

		const data = await request.formData();
		const token = data.get("token");

		if (token == null) {
			logger.error("token missing from request");
			return fail(400, { reason: "invalid token" });
		}

		const res = await fetch("/api/login", {
			method: "POST",
			headers: buildHeadersAuthless(true),
			body: JSON.stringify({ token }),
		});

		if (!res.ok) {
			logger.error({ response: res }, "login failure");
			invalidateCookie(cookies);
			return fail(400, { reason: "invalid token" });
		}

		const body: App.ApiJsonShape = await res.json();

		logger.info({ body: body.data }, "new session created");
		setCookie(cookies, body.data.session);
		return { ...body.data };
	},
} satisfies Actions;
