import type {
	ChannelEntry,
	ChatterEntry,
	Entry,
	PaginatedRequest,
	PaginatedResponse,
	ScoreWindows,
} from "$lib/types";
import type { RequestEvent } from "@sveltejs/kit";
import {
	capitalize,
	clamp,
	intoUntypedEntry,
	strToNum,
	type UntypedEntry,
} from ".";
import { logger } from "$lib/observability/server/logger.svelte";
import { traced } from "$lib/observability/server/tracing";
import { Rh } from "$lib/utils/route";

export class FetchUtil {
	@traced()
	async fetchLeaderboard(
		{ fetch }: Partial<RequestEvent>,
		variant: "channel" | "chatter",
		pagination: PaginatedRequest
	): Promise<PaginatedResponse> {
		let url = new URL(`${Rh.proto}://${Rh.api}/${variant}/leaderboard`);

		logger.info({ pageinfo: { ...pagination } }, "pagination");

		// const scoreLimit;
		// url.searchParams.append("score_limit", String(scoreLimit));

		const limit = strToNum(pagination.limit) || 15;
		const page = strToNum(pagination.page) || 0;

		if (limit) url.searchParams.append("limit", String(limit));
		if (page) url.searchParams.append("page", String(clamp(page - 1, 0)));

		logger.info({ url }, "[API] performing leaderboard fetch");

		const response = await fetch!(url, { method: "GET" });

		logger.info({ response }, "[API] query response");

		if (!response.ok) {
			// TODO check this
			//   the server SHOULD return some error info on error in its
			//   JSON body (i think lmfao but who KNOWS)
			logger.error({ response }, "[API] received error response");
		}

		const body = (await response.json()) as PaginatedResponse;
		logger.info({ leaderboard: { leaderboard: body, variant } });

		if (body.total_pages < Number(pagination.page)) {
			logger.warn(
				{ requestPage: pagination.page, totalPages: body.total_pages },
				"[API] requested non-existent page, using fallback of 'totalPages - 1'"
			);

			pagination.page = String(body.total_pages);
			return this.fetchLeaderboard({ fetch }, variant, pagination);
		}

		return body;
	}

	@traced()
	async fetchSingle(
		{ fetch }: Partial<RequestEvent>,
		variant: "channel" | "chatter",
		identVariant: "id" | "login",
		ident: string,
		pagination: PaginatedRequest
	): Promise<PaginatedResponse<UntypedEntry>> {
		const url = new URL(
			`${Rh.proto}://${Rh.api}/${variant}/by-${identVariant}/${ident}`
		);

		url.searchParams.set("score_limit", pagination.scoreLimit!);

		const scorePage = strToNum(pagination.scorePage!)!;
		url.searchParams.append("score_page", String(clamp(scorePage - 1, 0)));

		logger.info(
			{ url: url.href },
			"[API] built URL for single channel score fetch"
		);

		const apiResponse = await fetch!(url, { method: "GET" });

		logger.info({ response: apiResponse }, "[API] query response");
		if (!apiResponse.ok) {
			logger.error({ response: apiResponse }, "[API] received error response");
		}
		const body = intoUntypedEntry({
			_tag: capitalize(variant),
			data: await apiResponse.json(),
		});

		const scoreLimit = strToNum(pagination.scoreLimit!);

		const response: PaginatedResponse<UntypedEntry> = {
			page: scorePage!,
			total_items: body.totalScores,
			total_pages: Math.ceil(body.totalScores / scoreLimit!),
			page_size: body.scores.length,
			items: [body],
		};

		return response;
	}

	@traced()
	async fetchWindowed(
		{ fetch }: Partial<RequestEvent>,
		variant: "channel" | "chatter",
		id: string
	): Promise<ScoreWindows> {
		const url = new URL(`${Rh.proto}://${Rh.api}/${variant}/windowed/${id}`);
		url.searchParams.set("variant", variant);

		logger.info({ url: url.href }, "querying for windowed scores");

		const apiResponse = await fetch!(url, { method: "GET" });
		logger.info({ response: apiResponse }, "[API] query response");
		if (!apiResponse.ok) {
			logger.error({ response: apiResponse }, "[API] received error response");
		}

		return await apiResponse.json();
	}
}

export const fetchUtil = new FetchUtil();
