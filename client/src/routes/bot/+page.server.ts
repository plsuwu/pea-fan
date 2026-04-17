import type { PageServerLoad } from "./$types";
import { Rh } from "$lib/utils/route";
import type { Logger } from "pino";

export const load: PageServerLoad = async ({ fetch, locals }) => {
	const logger = locals.logger;
	const botStatus = await fetchBotEnabledBroadcasters(fetch, logger);

	return { botStatus };
};

type ChannelBotStatus = {
	id: string;
	enabled: boolean;
	login: string;
	name: string;
	color: string;
	image: string;
};

async function fetchBotEnabledBroadcasters(
	fetch: typeof globalThis.fetch,
	logger: Logger
): Promise<ChannelBotStatus[]> {
	const uri = `${Rh.apiv1}/channel/bot-state`;
	const childLogger = logger.child({ url: uri });

	try {
		const res = await fetch(uri, {
			method: "GET",
		});

		if (!res.ok) {
			childLogger.error({ status: res.status }, "failed to fetch bot state");
			return new Array();
		}

		const body = await res.json();
		return body.data;
	} catch (err) {
		childLogger.error({ error: err }, "error while fetching bot state");
		return new Array();
	}
}
