export const MODE_COOKIE_NAME = "mode-preference" as const;
const MAX_AGE = 60 * 60 * 24 * 365;

export function getParentDomain(): string {
	const parts = window.location.hostname.split(".");
	if (parts.length >= 2) {
		const parent = "." + parts.slice(-2).join(".");

		console.log("parent domain:", parent);
		return parent;
	}

	return window.location.hostname;
}

export function setModeCookie(mode: string) {
	document.cookie = `${MODE_COOKIE_NAME}=${mode}; domain=${getParentDomain()}; path=/; max-age=${MAX_AGE};`;
}

export function getModeCookie(): string | null {
	const match = document.cookie.match(
		new RegExp(`(?:^|; )${MODE_COOKIE_NAME}=([^;]*)`)
	);

	console.log(match ? match[1] : "");
	return match ? match[1] : null;
}
