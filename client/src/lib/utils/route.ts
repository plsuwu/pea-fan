import { env } from "$env/dynamic/public";
import { channelCache } from "$lib/observability/server/cache.svelte";
import type { RequestEvent } from "@sveltejs/kit";
import { isIpAddr } from ".";
import { logger } from "$lib/observability/server/logger.svelte";


export class Rh {
	private static readonly dev =
		env.PUBLIC_NODE_ENV === "development" ||
		env.PUBLIC_NODE_ENV === "staging" ||
		env.PUBLIC_NODE_ENV == null;

	private static readonly _proto = Rh.dev ? env.PUBLIC_USE_PROTO : "https";
	private static readonly _base = Rh.dev ? "piss.local" : "rat.moe";
	// private static readonly _api = "api.rat.moe";
	// private static readonly _apiProto = Rh.dev ? "http" : "https";

	// public static readonly apiBase = "https://api.rat.moe";
    public static readonly apiBase = "http://api.localhost:8080";

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

	// static get api(): string {
	// 	return this._api;
	// }
	//
	// static get apiProto(): string {
	// 	return this._apiProto;
	// }

	static get base(): string {
		return this._base;
	}
}
