import { fail, redirect, type Actions } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie, setCookie } from "$lib/server";
import type { PageServerLoad } from "../$types";
import { env } from "$env/dynamic/private";
import {
	AdminRateLimiter,
	clientIsRateLimited,
} from "$lib/server/rate-limit.svelte";
import { error } from "@sveltejs/kit";

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
		const childLogger = logger.child({
			client: locals.client,
			isRateLimited: locals.rateLimited,
			request,
		});

		if (
			locals.rateLimited ||
			clientIsRateLimited(locals.client.cfconnecting) ||
			!AdminRateLimiter.consume(locals.client.cfconnecting)
		) {
			childLogger.warn("[ADMIN_LOGIN] FAIL_RATE_LIMITED");
			fail(429, { reason: "rate_limit" });
		}

		const data = await request.formData();
		const token = data.get("token");

		if (!token) {
			childLogger.error("[ADMIN_LOGIN] FAIL_MISSING_TOKEN");
			fail(400, { reason: "invalid" });
		}

		const res = await fetch("/api/verify-totp", {
			method: "POST",
			headers: { "content-type": "application/json" },
			body: JSON.stringify({ token }),
		});

		const body = await res.json();

		if (!body.valid) {
			childLogger.error(
				{ valid: body.valid, session: body.session },
				"[ADMIN_LOGIN] FAIL_INVALID_TOKEN"
			);
			invalidateCookie(cookies);
			fail(400, { reason: "invalid" });
		}

		childLogger.info(
			{ valid: body.valid, session: body.session },
			"[ADMIN_LOGIN] PASS_TOKEN_OK"
		);

		setCookie(cookies, body.session);
		return { ...body };
	},
} satisfies Actions;
