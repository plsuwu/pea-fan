import { env } from "$env/dynamic/public";
import { channelCache } from "$lib/observability/server/cache.svelte";
import type { RequestEvent } from "@sveltejs/kit";
import { isIpAddr } from ".";
import { logger } from "$lib/observability/server/logger.svelte";

/**
 * Rh => `RouteHandler`
 */
export class Rh {
	private static readonly dev =
		env.PUBLIC_NODE_ENV === "development" || env.PUBLIC_NODE_ENV == null;

	private static readonly _proto = Rh.dev ? "http" : "https";
	private static readonly _base = Rh.dev ? "piss.local" : "rat.moe";
	private static readonly _api = Rh.dev ? "localhost:8080" : "api.rat.moe";

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

	static get api(): string {
		return this._api;
	}

	static get base(): string {
		return this._base;
	}
}
