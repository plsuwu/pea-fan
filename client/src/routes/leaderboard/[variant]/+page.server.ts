import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "./$types";
import { isValidVariant } from "$lib/utils";
import { fetchUtil } from "$lib/utils/fetching";

function getPageParamUrl(route: string) {
	const url = new URL(route);
	url.searchParams.set("page", "1");

	return url;
}

export const load: PageServerLoad = async ({
	locals,
	fetch,
	params,
	route,
	url
}) => {
	if (locals.channel) {
		locals.logger.warn(
			{ variant: params.variant, route: route.id },
			"global leaderboard routes invalid while on tenant"
		);
		redirect(302, "/");
	}

	if (!isValidVariant(params.variant)) {
		locals.logger.warn(
			{ variant: params.variant, route: route.id },
			`using fallback route '/leaderboard/channel' (invalid URN variant '${params.variant}')`
		);

		const newRoute = route.id.replace("[variant]", "channel");
		const newUrl = getPageParamUrl(newRoute);

		redirect(302, newUrl.href);
	}

	let { limit, page } = Object.fromEntries(url.searchParams);
	if (!page) {
		url.searchParams.set("page", "1");
		redirect(302, url);
	}

	if (params.variant === "channel") {
		limit = "50";
	}

	const result = await fetchUtil.fetchLeaderboard(
		{ fetch },
		params.variant as "channel" | "chatter",
		{ limit, page }
	);

	locals.logger.debug(
		{
			pagination_metadata: {
				curr_page: result.page,
				total_pages: result.total_pages,
				page_size: result.page_size,
				total_items: result.total_items,
				items_len: result.items.length
			}
		},
		"RX_API_RESPONSE"
	);

	return {
		leaderboardData: result,
		leaderboardVariant: params.variant
	};
};
