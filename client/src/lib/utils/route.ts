import { env } from "$env/dynamic/public";
import { channelCache } from "$lib/observability/server/cache.svelte";
import type { RequestEvent } from "@sveltejs/kit";
import { isIpAddr } from ".";
import { logger } from "$lib/observability/server/logger.svelte";

// Should be 'static' enough that we can load this before `Rh` is constructed, right??
export const API_BASE_HOST = env.PUBLIC_API_BASE_URL ?? "https://api.piss.fan";

export class Rh {
	private static readonly dev =
		env.PUBLIC_NODE_ENV === "development" ||
		env.PUBLIC_NODE_ENV === "staging" ||
		env.PUBLIC_NODE_ENV == null;

	private static readonly _proto = Rh.dev ? env.PUBLIC_USE_PROTO : "https";
	private static readonly _base = Rh.dev ? "piss.local" : "piss.fan";

	public static readonly apiv1 = `${API_BASE_HOST}/api/v1`;

	public static readonly apiPubChannel = `${Rh.apiv1}/channel`;
	public static readonly apiPubChatter = `${Rh.apiv1}/chatter`;
	public static readonly apiAdmin = `${Rh.apiv1}/_admin`;

	public static readonly apiHealth = `${Rh.apiv1}/checkhealth`;
	public static readonly apiSearch = `${Rh.apiv1}/search`;

	constructor() {
		console.log(Rh._proto);
	}

	static deriveBase(host: string) {
		const parts = host.split(".");
		if (parts.length > 1 && parts[1].includes("localhost")) {
			return parts.slice(1).join(".");
		}

		return parts.length > 2 && !isIpAddr(host)
			? parts.slice(1).join(".")
			: host;
	}

	static getTenantedURL(login: string, host?: string): URL {
		const base = host ? Rh.deriveBase(host) : Rh._base;
		return new URL(`${Rh._proto}://${login}.${base}`);
	}

	static async reroutable(_: RequestEvent, channel: string) {
		logger.debug({ requestHost: _.url.host }, "[SHOULD REROUTE] checking");

		const isValid = await channelCache.exists(channel);
		return isValid;
	}

	static get proto(): string {
		return this._proto;
	}

	static get base(): string {
		return this._base;
	}
}
