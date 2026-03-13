import { fail, redirect, type Actions } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { invalidateCookie, setCookie } from "$lib/server";
import type { PageServerLoad } from "../$types";
import { ADMIN_SESSION_TOKEN } from "$env/static/private";

export const load: PageServerLoad = async ({ cookies, locals, url }) => {
	const hasToken = cookies.get(ADMIN_SESSION_TOKEN);
	if (hasToken) {
		locals.logger.warn(
			{ token: hasToken },
			"session token present, redirecting instead"
		);

		redirect(302, "/admin");
	}
    
    const redirectReason = url.searchParams.get('err') || null;
    console.log("search params:", redirectReason);

    return {
        redirectReason,
    }
};

export const actions = {
	default: async ({ cookies, request, fetch }) => {
		const data = await request.formData();
		const token = data.get("token");

		if (!token) {
			logger.error("missing token");
			fail(400, { reason: "invalid" });
		}

		const res = await fetch("/api/verify-totp", {
			method: "POST",
			headers: { "content-type": "application/json" },
			body: JSON.stringify({ token }),
		});

		const body = await res.json();
		logger.debug(
			{ valid: body.valid, session: body.session },
			"login handler result"
		);

		if (!body.valid) {
			invalidateCookie(cookies);
			fail(400, { reason: "invalid" });
		}

		setCookie(cookies, body.session);
		return { ...body };
	},
} satisfies Actions;
