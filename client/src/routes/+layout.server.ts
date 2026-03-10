import type { LayoutServerLoad } from "./$types";
import { fetchUtil } from "$lib/utils/fetching";
import { MODE_COOKIE_NAME } from "$lib/utils/mode-cookie";
import type { PaginatedRequest } from "$lib/types";
import { redirect } from "@sveltejs/kit";

export const load: LayoutServerLoad = async ({
	url,
	locals,
	fetch,
	cookies,
}) => {
	const modePreference = cookies.get(MODE_COOKIE_NAME) ?? null;

	if (locals.channel) {
		const pagination = buildSingleChannelParams({ url });
		const leaderboard = await fetchUtil.fetchSingle(
			{ fetch },
			"channel",
			"login",
			locals.channel,
			pagination
		);

		const scoreWindows = await fetchUtil.fetchWindowed(
			{ fetch },
			"channel",
			leaderboard.items[0].id
		);

		return {
			leaderboard,
			scoreWindows,
			channel: locals.channel,
			modePreference,
		};
	}

	return {
		leaderboard: null,
        scoreWindows: null,
		channel: null,
		modePreference,
	};
};

function buildSingleChannelParams({ url }: { url: URL }) {
	let { score_limit, score_page } = Object.fromEntries(url.searchParams);

	if (!score_limit) score_limit = "25";
	if (!score_page) score_page = "1";

	const pagination: PaginatedRequest = {
		scoreLimit: score_limit,
		scorePage: score_page,
		// we shouldn't actually care about these params at all, but we need to provide
		// them to avoid throwing errors somewhere - not IDEAL but it is what it is
		limit: "0",
		page: "0",
	};

	return pagination;
}
