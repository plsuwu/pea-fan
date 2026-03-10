export const MODE_COOKIE_NAME = "mode-preference" as const;
const MODE_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

function getParentDomain(): string {
	const parts = window.location.hostname.split(".");
	if (parts.length >= 2) {
		const parent = "." + parts.slice(-2).join(".");
        return parent;
	}
	return window.location.hostname;
}

export function setModeCookie(mode: string) {
	const domain = getParentDomain();
	document.cookie = `${MODE_COOKIE_NAME}=${mode}; domain=.lvh.me; path=/; max-age=${MODE_COOKIE_MAX_AGE}; SameSite=Lax`;
}

export function getModeCookie(): string | null {
	const match = document.cookie.match(
		new RegExp(`(?:^|; )${MODE_COOKIE_NAME}=([^;]*)`)
	);
	return match ? match[1] : null;
}
