import { redirect } from "@sveltejs/kit";
import type { PageServerLoad, RouteParams } from "./$types";
import { clamp, isValidVariant, strToNum } from "$lib/utils";
import { Rh } from "$lib/utils/route";

function getPageParamUrl(route: string) {
	const url = new URL(route);
	url.searchParams.set("page", "1");

	return url;
}

function getSanitizedParams(
	url: URL,
	params: RouteParams
): { limit: string; page: string } | null {
	let { limit, page } = Object.fromEntries(url.searchParams);
	if (!page) {
		return null;
	}

	if (params.variant === "channel") {
		limit = "100";
	}

	const sanitizedLimit = strToNum(limit) || 15;
	const sanitizedPage = strToNum(page) || 0;
	return {
		limit: String(sanitizedLimit),
		page: String(clamp(sanitizedPage - 1, 0)),
	};
}

export const load: PageServerLoad = async ({
	locals,
	fetch,
	params,
	route,
	url,
}) => {
	// don't display a leaderboard if we are on tenant route
	if (locals.channel) {
		redirect(302, "/");
	}

	// ensure we are trying to view a valid leaderboard variant 
    // (i.e. "channel" or "chatter")
	if (!isValidVariant(params.variant)) {
		locals.logger.warn(
			{ variant: params.variant, route: route.id },
			`${params.variant} invalid leaderboard variant`
		);

		const newRoute = route.id.replace("[variant]", "channel");
		const newUrl = getPageParamUrl(newRoute);
		redirect(302, newUrl.href);
	}

	const urlSearchParams = getSanitizedParams(url, params);
	if (!urlSearchParams) {
		url.searchParams.set("page", "1");
		redirect(302, url);
	}

	const fetchUrl = new URL(`${Rh.apiv1}/${params.variant}/leaderboard`);
	fetchUrl.searchParams.set("page", urlSearchParams.page);
	fetchUrl.searchParams.set("limit", urlSearchParams.limit);

	try {
		const res = await fetch(fetchUrl, {
			method: "GET",
		});

		const body = await res.json();
		if (!res.ok) {
			locals.logger.error(
				{ status: res.status, error: body.error },
				"failed to fetch variant leaderboard"
			);

			return {
				leaderboardData: null,
				leaderboardVariant: params.variant,
			};
		}

		const result = body.data;
		// locals.logger.trace(
		// 	{
		// 		currentPage: result.page,
		// 		totalPages: result.total_pages,
		// 		totalItems: result.total_items,
		// 	},
		// 	"retrieved variant leaderboard data"
		// );

		return {
			leaderboardData: result,
			leaderboardVariant: params.variant,
		};
	} catch (err) {
		locals.logger.error(
			{ error: err },
			"internal error during leaderboard fetch"
		);

		return {
			leaderboardData: null,
			leaderboardVariant: params.variant,
		};
	}
};
