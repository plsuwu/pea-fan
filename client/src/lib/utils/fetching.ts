import type { Entry, PaginatedRequest, PaginatedResponse } from "$lib/types";
import type { RequestEvent } from "@sveltejs/kit";
import { clamp, strToNum } from ".";
import { URLS } from "$lib";
import { logger } from "$lib/observability/server/logger.svelte";
import { traced } from "$lib/observability/server/tracing";

export class FetchUtil {
	public api: string;
	public base: string;
	public proto: string;

	constructor() {
		const { api, base, proto } = URLS();
		this.api = api;
		this.base = base;
		this.proto = proto;
	}

	@traced()
	async fetchLeaderboard(
		{ fetch }: Partial<RequestEvent>,
		variant: "channel" | "chatter",
		pagination: PaginatedRequest
	): Promise<PaginatedResponse> {
		let url = new URL(`${this.proto}://${this.api}/${variant}/leaderboard`);

		const limit = strToNum(pagination.limit);
		const page = strToNum(pagination.page);

		if (limit) url.searchParams.append("limit", String(limit));
		if (page) url.searchParams.append("page", String(clamp(page - 1, 0)));

		const response = await fetch!(url, { method: "GET" });

		if (!response.ok) {
			// the server SHOULD return some error info on error in its
			// JSON body (i think lmfao)
			logger.error({ response }, "received error response from API");
		}

		const body = (await response.json()) as PaginatedResponse;
		logger.info({ leaderboard: { leaderboard: body, variant } });

		if (body.total_pages < Number(pagination.page)) {
			logger.warn(
				{ requestPage: pagination.page, totalPages: body.total_pages },
				"requested non-existent page, using fallback of 'totalPages - 1'"
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
		// pagination: PaginatedRequest
	): Promise<Entry> {
		const url = new URL(
			`${this.proto}://${this.api}/${variant}/by-${identVariant}/${ident}`
		);

		const response = await fetch!(url, { method: "GET" });
		if (!response.ok) {
			logger.error({ response }, "received error response from API");
		}

		const body = (await response.json()) as Entry;
		logger.info({ entry: body });

		return body;
	}
}

export const fetchUtil = new FetchUtil();
