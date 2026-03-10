import {
	PUBLIC_PROD_API,
	PUBLIC_PROD_BASE,
	PUBLIC_PROD_PROTO,
	PUBLIC_DEVL_API,
	PUBLIC_DEVL_BASE,
	PUBLIC_DEVL_PROTO,
} from "$env/static/public";
import { channelCache } from "$lib/observability/server/cache.svelte";
import type { RequestEvent } from "@sveltejs/kit";

/**
 * Rh => `RouteHandler`
 */
export class Rh {
	private static readonly dev = import.meta.env.DEV;

	private static readonly _proto = Rh.dev
		? PUBLIC_DEVL_PROTO
		: PUBLIC_PROD_PROTO;
	private static readonly _api = Rh.dev ? PUBLIC_DEVL_API : PUBLIC_PROD_API;
	private static readonly _base = Rh.dev ? PUBLIC_DEVL_BASE : PUBLIC_PROD_BASE;

	static deriveBase(host: string) {
		const parts = host.split(".");
		if (parts.length > 1 && parts[1].includes("localhost")) {
			return parts.slice(1).join(".");
		}

		return parts.length > 2 ? parts.slice(1).join(".") : host;
	}

	static getTenantedURL(login: string, host?: string): URL {
		const base = host ? Rh.deriveBase(host) : Rh._base;
		return new URL(`${Rh._proto}://${login}.${base}`);
	}

	static async reroutable(_: RequestEvent, channel: string) {
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
