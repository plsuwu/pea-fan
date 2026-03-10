import { ADMIN_COOKIE_KEY } from "$env/static/private";
import { type Actions } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { getTokenHash } from "$lib/utils";

function setCookieValue(verified: boolean, hashedTokenHex: string) {
	return verified ? hashedTokenHex : "";
}

export const actions = {
	login: async ({ cookies, request, fetch }) => {
		const data = await request.formData();
		const tokenRaw = (data.get("token") as string) ?? "AAAAA";

		logger.info({ tokenRaw }, "[ACTION] submitted login token");

        const token = getTokenHash(tokenRaw);
        logger.info({ token }, "[ACTION] hashed token"); 

		const res = await fetch("/api/verify-token", {
			method: "POST",
			body: JSON.stringify({ token })
		});

		const { verified } = await res.json();
		logger.debug({ verified: verified }, "[ACTION] login handler result");

		const cookieValue = setCookieValue(verified, token);

		cookies.set(ADMIN_COOKIE_KEY, cookieValue, {
			path: "/",
			httpOnly: true,
			sameSite: "lax"
		});

		return { verified };
	}
} satisfies Actions;
