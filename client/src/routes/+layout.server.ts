import type { LayoutServerLoad } from "./$types";
import { fetchUtil } from "$lib/utils/fetching";
import { MODE_COOKIE_NAME } from "$lib/utils/mode-cookie.svelte";
import type {
	PaginatedRequest,
	PaginatedResponse,
	PaginationData,
} from "$lib/types";
import {
	intoParentEntry,
	type UntypedEntry,
	type UntypedSubEntry,
} from "$lib/utils";

export const load: LayoutServerLoad = async ({
	locals,
	cookies,
	fetch,
	url,
}) => {
	const modePreference = cookies.get(MODE_COOKIE_NAME) ?? null;
	const announcementClearToken = cookies.get("seen-announcement") || null;

	if (!locals.channel) {
		return {
			leaderboard: null,
			scoreWindows: null,
			channel: null,
			modePreference,
			announcementClearToken,
			announcement: "test announcement, hello, hello 123",
		};
	}

	const pagination = buildSingleChannelParams({ url });
	const rawChannelData = await fetchUtil.fetchSingle(
		{ fetch },
		"channel",
		"login",
		locals.channel,
		pagination
	);

	const scoreWindows = await fetchUtil.fetchWindowed(
		{ fetch },
		"channel",
		rawChannelData.items[0].id
	);

	return {
		channelData: parseChannelData(rawChannelData),
		paginationData: parsePaginationData(rawChannelData),
		channel: locals.channel,
		modePreference,
		scoreWindows,
	};
};

function parsePaginationData(data: PaginatedResponse<any>): PaginationData {
	const { page, total_items, total_pages, page_size } = data;
	return {
		currentPage: page,
		totalItems: total_items,
		itemsPerPage: page_size,
		totalPages: total_pages,
	};
}

function parseChannelData(
	data: PaginatedResponse<UntypedEntry>
): UntypedEntry<UntypedEntry> {
	const [channelItems] = data.items;
	const scores = channelItems.scores.map((entry: UntypedSubEntry) =>
		intoParentEntry(entry)
	);

	return { ...channelItems, scores };
}

function buildSingleChannelParams({ url }: { url: URL }) {
	let { score_limit, score_page } = Object.fromEntries(url.searchParams);

	if (!score_limit) score_limit = "25";
	if (!score_page) score_page = "1";

	const pagination: PaginatedRequest = {
		scoreLimit: score_limit,
		scorePage: score_page,
		limit: "0",
		page: "0",
	};

	return pagination;
}
