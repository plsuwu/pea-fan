import { page as _page } from "$app/state";

const BASE_TITLE = "piss fan";

export const updatePageTitle = (
	page: typeof _page,
	channel?: string | null
) => {
	if (channel != null) {
		return `${channel} | ${BASE_TITLE}`;
	}

	const raw = page.url.pathname;
	if (raw !== "/admin" && raw !== "/admin/login") {
		const pageNumberParam = page.url.searchParams.get("page");
		const currentPage = pageNumberParam ? `| page ${pageNumberParam}` : "";

		const currentPath = raw
			.trim()
			.split("/")
			.filter((p) => p !== "");

		if (currentPath[1] != null) {
			return `${currentPath[1]}s ${currentPage} ${BASE_TITLE}`;
		}
	}
};
