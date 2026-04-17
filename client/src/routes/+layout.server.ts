import type { LayoutServerLoad } from "./$types";
import { logger as serverLogger } from "$lib/observability/server/logger.svelte";
import { MODE_COOKIE_NAME } from "$lib/utils/mode-cookie.svelte";
import { sha256 } from "@oslojs/crypto/sha2";
import { encodeHexLowerCase } from "@oslojs/encoding";
import type { Chatter } from "$lib/types";
import { Rh } from "$lib/utils/route";
import {
	clamp,
	intoParentEntry,
	intoUntypedEntry,
	strToNum,
	type UntypedSubEntry,
} from "$lib/utils";
import { announcementCache } from "$lib/observability/server/cache.svelte";

function makeSha256Hash(str: string): string {
	const bytes = new TextEncoder().encode(str);
	return encodeHexLowerCase(sha256(bytes));
}

async function fetchLiveBroadcasters(
	fetch: typeof globalThis.fetch,
	logger: typeof serverLogger
) {
	const uri = `${Rh.apiv1}/channel/live`;
	const childLogger = logger.child({
		url: uri,
	});

	try {
		const res = await fetch(uri, { method: "GET" });
		if (!res.ok) {
			childLogger.error("failed to retrieve live broadcaster list");
			return null;
		}

		const body = await res.json();
		return body.data;
	} catch (err) {
		childLogger.error({ error: err }, "failed during live broadcaster fetch");
		return null;
	}
}

export type Announcement = {
	content: string | null;
	hash: string | null;
};

async function fetchAnnouncement(
	_logger: typeof serverLogger,
	seenAnnounce: string | null
): Promise<Announcement & { seen: boolean }> {
	const announcement = {
		...(await announcementCache.getAnnouncement()),
		seen: false,
	};

	if (announcement.hash && announcement.hash === seenAnnounce) {
		announcement.seen = true;
	}

	return announcement;
}

function defaultLayoutData(
	liveBroadcasters: Chatter[],
	modePreference: string | null,
	announcement: Announcement
) {
	return {
		leaderboard: null,
		scoreWindows: null,
		channel: null,
		liveBroadcasters,
		modePreference,
		announcement,
	};
}

export const load: LayoutServerLoad = async ({
	locals,
	cookies,
	fetch,
	url,
}) => {
	const modePreference = cookies.get(MODE_COOKIE_NAME) ?? null;
	const announcement: Announcement = await fetchAnnouncement(
		locals.logger,
		cookies.get("seen-announcement") || null
	);

	const liveBroadcasters = await fetchLiveBroadcasters(fetch, locals.logger);
	const baseLayoutData = defaultLayoutData(
		liveBroadcasters,
		modePreference,
		announcement
	);

	if (!locals.channel) {
		return baseLayoutData;
	}

	const { channelData, paginationData } = await getChannelLeaderboard(
		fetch,
		url,
		locals
	);

	if (channelData == null || paginationData == null || channelData.id == null) {
		return { ...baseLayoutData, channel: locals.channel };
	}

	const scoreWindows = await getPeriodicChannelData(
		fetch,
		locals,
		channelData.id
	);

	if (scoreWindows == null) {
		return {
			...baseLayoutData,
			channel: locals.channel,
			channelData,
			paginationData,
		};
	}

	return {
		...baseLayoutData,
		channel: locals.channel,
		channelData,
		paginationData,
		scoreWindows,
	};
};

async function getPeriodicChannelData(
	fetch: typeof globalThis.fetch,
	locals: App.Locals,
	id: string
) {
	const fetchUrl = new URL(`${Rh.apiv1}/channel/windowed/${id}`);
	fetchUrl.searchParams.set("variant", "channel");

	const logger = locals.logger.child({
		url: fetchUrl,
	});

	try {
		const res = await fetch(fetchUrl, {
			method: "GET",
		});

		if (!res.ok) {
			const body = await res.json();
			logger.error(
				{ status: res.status, error: body.error },
				"failed to fetch periodic data"
			);

			return null;
		}

		const body = await res.json();
		return body.data;
	} catch (err) {
		logger.error(
			{ error: err },
			"internal error during single channel leaderboard fetch"
		);

		return null;
	}
}

async function getChannelLeaderboard(
	fetch: typeof globalThis.fetch,
	url: URL,
	locals: App.Locals
) {
	const { scoreLimit, scorePage } = buildSingleChannelParams(url);
	const pagination = {
		scoreLimit: String(scoreLimit),
		scorePage: String(clamp(scorePage - 1, 0)),
	};
	const fetchUrl = new URL(`${Rh.apiv1}/channel/by-login/${locals.channel}`);

	// fetchUrl.searchParams.set("page", "0");
	// fetchUrl.searchParams.append("limit", "0");

	fetchUrl.searchParams.set("score_limit", pagination.scoreLimit!);
	fetchUrl.searchParams.set("score_page", pagination.scorePage!);

	const logger = locals.logger.child({
		url: fetchUrl,
	});

	try {
		logger.info("fetching chatter leaderboard for channel");
		const res = await fetch(fetchUrl, {
			method: "GET",
		});

		if (!res.ok) {
			const body = await res.json();
			logger.error(
				{ status: res.status, error: body.error },
				"failed to fetch single channel leaderboard"
			);

			// return null on non-ok response
			return {
				channelData: null,
				paginationData: null,
			};
		}

		const body = await res.json();
		const entryBody = intoUntypedEntry({
			_tag: "Channel",
			data: body.data,
		});

		const scores = entryBody.scores.map((entry: UntypedSubEntry) =>
			intoParentEntry(entry)
		);

		const channelData = { ...entryBody, scores };
		const paginationData = {
			currentPage: scorePage,
			totalItems: entryBody.totalScores,
			totalPages: Math.ceil(entryBody.totalScores / scoreLimit),
			itemsPerPage: scores.length,
		};

		logger.info("retrieved single score data");

		// return channel data on success
		return {
			channelData,
			paginationData,
		};
	} catch (err) {
		logger.error(
			{ error: err },
			"internal error during single channel leaderboard fetch"
		);

		// return null on error throw
		return {
			channelData: null,
			paginationData: null,
		};
	}
}

function buildSingleChannelParams(url: URL) {
	let { score_limit, score_page } = Object.fromEntries(url.searchParams);

	if (!score_limit) score_limit = "10";
	if (!score_page) score_page = "1";

	const sanitizedLimit = strToNum(score_limit) || 10;
	const sanitizedPage = strToNum(score_page) || 1;

	const pagination = {
		scoreLimit: sanitizedLimit,
		scorePage: sanitizedPage,
	};

	return pagination;
}
