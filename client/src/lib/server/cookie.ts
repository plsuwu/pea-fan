import { ADMIN_SESSION_TOKEN } from "$env/static/private";
import type { Cookies } from "@sveltejs/kit";

export function invalidateCookie(cookies: Cookies): void {
	cookies.delete(ADMIN_SESSION_TOKEN, {
		sameSite: "lax",
		secure: false,
		path: "/",
	});
}

export function setCookie(cookies: Cookies, token: string): void {
	const MAX_AGE = 100 * 60 * 60 * 24 * 14;

	cookies.set(ADMIN_SESSION_TOKEN, token, {
		path: "/",
		sameSite: "lax",
		maxAge: MAX_AGE,
		secure: false,
	});
}
