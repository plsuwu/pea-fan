import { URLS } from "$lib";
import { channelCache } from "$lib/observability/server/cache.svelte";
import { traced } from "$lib/observability/server/tracing";
import type { RequestEvent } from "@sveltejs/kit";

export class RouteUtil {
	public readonly proto: string;
	public readonly api: string;
	public readonly base: string;

	constructor() {
		const { proto, base, api } = URLS();
		this.proto = proto;
		this.base = base;
		this.api = api;
	}

	@traced()
	async reroutable(event: RequestEvent, channel: string) {
		const isValid = await channelCache.exists(`#${channel}`);
		event.locals.logger.debug(
			{ valid: isValid, channelName: channel },
			"route validation result"
		);

		return isValid;
	}
    
    // this is primarily used in the browser and i dont think we REALLY care to do any 
    // logging/tracing on this function
	getTenantHref(login: string): URL {
		return new URL(`${this.proto}://${login}.${this.base}`);
	}
}

export const rtUtil = new RouteUtil();
