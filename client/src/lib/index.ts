// import {
// 	PUBLIC_DEVL_API,
// 	PUBLIC_DEVL_BASE,
// 	PUBLIC_DEVL_PROTO,
// 	PUBLIC_PROD_API,
// 	PUBLIC_PROD_BASE,
// 	PUBLIC_PROD_PROTO
// } from "$env/static/public";
// import type { RequestEvent } from "@sveltejs/kit";
// import { channelCache } from "./observability/server/cache.svelte";
// import { traced } from "./observability/server/tracing";
//
// class _RouteHandler {
// 	private readonly _protocol;
// 	private readonly _api;
// 	private readonly _base;
//
// 	constructor(dev: boolean) {
// 		console.log("dev:", dev);
// 		if (dev) {
// 			this._api = PUBLIC_PROD_API;
// 			this._base = PUBLIC_PROD_BASE;
// 			this._protocol = PUBLIC_PROD_PROTO;
// 		} else {
// 			this._api = PUBLIC_DEVL_API;
// 			this._base = PUBLIC_DEVL_BASE;
// 			this._protocol = PUBLIC_DEVL_PROTO;
// 		}
// 	}
//
// 	deriveBase(host: string) {
// 		const parts = host.split(".");
// 		return parts.length > 1 ? parts.slice(1).join(".") : host;
// 	}
//
// 	@traced()
// 	async reroutable(_: RequestEvent, channel: string) {
// 		return await channelCache.exists(`#${channel}`);
// 	}
//
// 	tenantHref(login: string, host?: string): URL {
// 		const base = host ? this.deriveBase(host) : this._base;
// 		return new URL(`${this._protocol}://${login}.${base}`);
// 	}
//
// 	get proto(): string {
// 		return this._protocol;
// 	}
//
// 	get api(): string {
// 		return this._api;
// 	}
//
// 	get base(): string {
// 		return this._base;
// 	}
// }
//
// export const RouteHandler = new _RouteHandler(import.meta.env.DEV);
//
// // export const URLS = (): { api: string; base: string; proto: string } => {
// // 	if (!import.meta.env.DEV) {
// // 		return {
// // 			api: PUBLIC_PROD_API,
// // 			base: PUBLIC_PROD_BASE,
// // 			proto: PUBLIC_PROD_PROTO
// // 		};
// // 	}
// //
// // 	return {
// // 		api: PUBLIC_DEVL_API,
// // 		base: PUBLIC_DEVL_BASE,
// // 		proto: PUBLIC_DEVL_PROTO
// // 	};
// // };
