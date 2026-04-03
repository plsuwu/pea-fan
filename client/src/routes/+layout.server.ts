import type { LayoutServerLoad } from "./$types";
import { logger as serverLogger } from "$lib/observability/server/logger.svelte";
import { fetchUtil } from "$lib/utils/fetching";
import { MODE_COOKIE_NAME } from "$lib/utils/mode-cookie.svelte";
import { sha256 } from "@oslojs/crypto/sha2";
import { encodeHexLowerCase } from "@oslojs/encoding";
import type {
	PaginatedRequest,
	PaginatedResponse,
	PaginationData,
} from "$lib/types";
import { Rh } from "$lib/utils/route";
import {
	intoParentEntry,
	type UntypedEntry,
	type UntypedSubEntry,
} from "$lib/utils";

const LIVE_BROADCASTERS = `${Rh.apiBase}/channel/live`;

function makeSha256Hash(str: string): string {
	const bytes = new TextEncoder().encode(str);
	return encodeHexLowerCase(sha256(bytes));
}

async function fetchLiveBroadcasters(
	fetch: typeof globalThis.fetch,
	logger: typeof serverLogger
) {
	const res = await fetch(LIVE_BROADCASTERS, {
		method: "GET",
	});

	if (res.status != 200) {
		logger.error(
			{ response: res, url: LIVE_BROADCASTERS },
			"[API]: FAIL_STATUS_RECV"
		);
		return null;
	}

	const body = await res.json();
	return body;
}

export const load: LayoutServerLoad = async ({
	locals,
	cookies,
	fetch,
	url,
}) => {
	const modePreference = cookies.get(MODE_COOKIE_NAME) ?? null;
	const announcementContent = `<div class='text-center>
    <div class='pt-1'>thanks for checking the new version out!!
        please <a href='/about#incorrect-scores'
            class='text-blue-500 hover:opacity-50'
        >
            click here
        </a>
        if your count doesnt seem right (or see the 'about' page later).
    </div>
</div>`;

	const announcement = {
		content: announcementContent,
		hash: makeSha256Hash(announcementContent),
	};
	const seenAnnouncement = cookies.get("seen-announcement") || null;
	const announcementClearToken = seenAnnouncement === announcement.hash;

	const liveBroadcasters = await fetchLiveBroadcasters(fetch, locals.logger);

    locals.logger.trace({ liveBroadcasters }, "LIVE BROADCASTERS");

	if (!locals.channel) {
		return {
			leaderboard: null,
			scoreWindows: null,
			channel: null,
            liveBroadcasters,
			modePreference,
			announcementClearToken,
			announcement,
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
		liveBroadcasters,
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
