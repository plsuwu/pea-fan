import { env } from "$env/dynamic/public";
import type { RequestEvent } from "@sveltejs/kit";
import { isIpAddr } from ".";
import { logger } from "$lib/observability/server/logger.svelte";
import { channelCache } from "$lib/caching";

export const BASE_HOST = env.PUBLIC_BASE_HOST ?? "piss.fan";
export const INTERN_API = env.PUBLIC_INTERNAL_API ?? "http://localhost:8080";
export const EXTERN_API = env.PUBLIC_EXTERNAL_API ?? "https://api.piss.fan";

export const TENANT_PLACEHOLDER = "__";

export type ApiRoute = (typeof API_ROUTE)[number];
export const API_ROUTE = [
    "auth",
	"checkhealth",
	"channel",
	"chatter",
	"search",
	"_admin",
] as const;

export interface Route {
	_devel: boolean;
	_apiBase: string;

	proto: string;
	host: string;
	baseRoutes: typeof API_ROUTE;

	// top-level API domain, e.g.
	//  - "https://api.piss.fan",
	//  - "http://localhost:8080",
	api: {
		internal: string;
		external: string;
	};
}

export class RouteManager implements Route {
	readonly _apiBase = "api/v1";
	readonly _devel = env.PUBLIC_NODE_ENV !== "production";

	readonly host = BASE_HOST;
	readonly baseRoutes = API_ROUTE;
	readonly proto = this._devel ? "http" : "https";

	readonly api = {
		internal: `${INTERN_API}/${this._apiBase}`,
		external: `${EXTERN_API}/${this._apiBase}`,
	};

	constructor() {
		// logger.trace({ self: this }, "BASE ROUTES IN ROUTE MANAGER");
	}

	public externApiUrl(at: ApiRoute, loc: string) {
		const uri = `${this.api.external}/${at}/${loc}`;
		// logger.debug({ route: uri }, "routeman constructed external uri");

		return uri;
	}

	public internApiUrl(at: ApiRoute, loc: string) {
		const uri = `${this.api.internal}/${at}/${loc}`;
		// logger.debug({ route: uri }, "routeman constructed internal uri");

		return uri;
	}

	deriveBase(host: string) {
		const parts = host.split(".");
		if (parts.length > 1 && parts[1].includes("localhost")) {
			const out = parts.slice(1).join(".");
			// logger.debug({ host: out }, "derived base localhost host");

			return out;
		}

		const out =
			parts.length > 2 && !isIpAddr(host) ? parts.slice(1).join(".") : host;
		// logger.debug({ host: out }, "derived base external host");

		return out;
	}

	getTenantedURL(login: string, host?: string): URL {
		const base = host ? this.deriveBase(host) : this.host;
		const res = new URL(`${this.proto}://${login}.${base}`);
		// logger.debug({ result: res }, "using tenanted base");

		return res;
	}

	getUntenantedURL(host?: string): URL {
		const base = host ? this.deriveBase(host) : this.host;
		const res = new URL(`${this.proto}://${base}`);
		// logger.trace({ result: res }, "using base");

		return res;
	}

	async reroutable(channel: string) {
		const channels = await channelCache.read();
		return channels.includes(channel);
	}
}

export const routeManager = new RouteManager();

// export class Rh {
// 	private static readonly dev =
// 		env.PUBLIC_NODE_ENV === "development" ||
// 		env.PUBLIC_NODE_ENV === "staging" ||
// 		env.PUBLIC_NODE_ENV == null;
//
//     public static readonly apiv1Local = "http://localhost:8080/api/v1";
//     public static readonly apiLocalAdmin = `${Rh.apiv1Local}/_admin`;
//
//
// 	private static readonly _proto = Rh.dev ? env.PUBLIC_USE_PROTO : "https";
// 	private static readonly _base = Rh.dev ? "piss.local" : "piss.fan";
//
// 	public static readonly apiv1 = `${API_BASE_HOST}/api/v1`;
//
// 	public static readonly apiPubChannel = `${Rh.apiv1}/channel`;
// 	public static readonly apiPubChatter = `${Rh.apiv1}/chatter`;
// 	public static readonly apiAdmin = `${Rh.apiv1}/_admin`;
//
// 	public static readonly apiHealth = `${Rh.apiv1}/checkhealth`;
// 	public static readonly apiSearch = `${Rh.apiv1}/search`;
//
// 	constructor() {
// 		console.log(Rh._proto);
// 	}
//
// 	static deriveBase(host: string) {
// 		const parts = host.split(".");
// 		if (parts.length > 1 && parts[1].includes("localhost")) {
// 			return parts.slice(1).join(".");
// 		}
//
// 		return parts.length > 2 && !isIpAddr(host)
// 			? parts.slice(1).join(".")
// 			: host;
// 	}
//
// 	static getTenantedURL(login: string, host?: string): URL {
// 		const base = host ? Rh.deriveBase(host) : Rh._base;
// 		return new URL(`${Rh._proto}://${login}.${base}`);
// 	}
//
// 	static async reroutable(_: RequestEvent, channel: string) {
// 		logger.debug({ requestHost: _.url.host }, "[SHOULD REROUTE] checking");
//
// 		const isValid = await channelCache.exists(channel);
// 		return isValid;
// 	}
//
// 	static get proto(): string {
// 		return this._proto;
// 	}
//
// 	static get base(): string {
// 		return this._base;
// 	}
// }
