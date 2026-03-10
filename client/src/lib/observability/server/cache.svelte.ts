import { Rh } from "$lib/utils/route";
import { logger } from "./logger.svelte";
import { Result } from "$lib/types/result/result";
import { traced } from "./tracing";
import {
	PUBLIC_DEVL_API,
	PUBLIC_DEVL_PROTO,
	PUBLIC_PROD_API,
	PUBLIC_PROD_PROTO,
} from "$env/static/public";

export type RawIrcInfo = {
	likely_missing: string[];
	full_list: string[];
	current_joins: string[];
};

export abstract class Cache<T extends string> {
	readonly url: string;
	readonly name: string;

	protected _data: Set<T> = new Set();
	protected _lastRefresh: number | null = null;
	protected _ttlMs: number = 300000; // 300 secs = 5 mins

	constructor(url: string, cacheName: string, ttl = 300000) {
		this.url = url;
		this.name = cacheName;
		// this._ttlMs = ttl;
		this._ttlMs = 1000;
	}

	@traced()
	public async refresh(method = "GET"): Promise<void> {
		if (!this.ttlElapsed()) {
			logger.debug({ cache: this }, "[CACHE] up to date");
			return;
		}

		logger.debug({ cache: this }, "[CACHE] running refresh");
		const response = await fetch(this.url, { method });
		if (!response.ok) {
			logger.error({ response }, "[CACHE] error during update");
			return;
		}

		(await Result.fromPromise(response.json())).match({
			Ok: (data) => {
				this.data = new Set(data as string[] as Iterable<T>);
				this._lastRefresh = Date.now();
			},
			Err: (err) => {
				logger.error(
					{ cache: this, error: err },
					"[CACHE] JSON response error"
				);

				throw err;
			},
		});

		logger.info({ cache: this }, "[CACHE] retrieved channel data");
		return;
	}

	@traced()
	public async exists(name: T): Promise<boolean> {
		await this.refresh();
		return this._data.has(name);
	}

	@traced()
	protected ttlElapsed(): boolean {
		const stale =
			this._lastRefresh == null || this._lastRefresh + this._ttlMs < Date.now();

		if (stale) {
			logger.warn("[CACHE] stale data: attempting refresh");
		}

		return stale;
	}

	// setters

	set data(names: Iterable<T>) {
		this._data = new Set(names);
	}

	set ttl(ttl: number) {
		this._ttlMs = ttl;
	}

	// getters

	get apiEndpoint(): string {
		return this.url;
	}

	get data(): Array<T> {
		return this._data.keys().toArray();
	}

	get ttl(): string {
		return `${this._ttlMs / 1000}s`;
	}

	get lastRefresh(): string {
		return this._lastRefresh
			? new Date(this._lastRefresh).toLocaleString()
			: "never_refreshed";
	}

	get nextRefresh(): string {
		return new Date(
			this._lastRefresh ? this._ttlMs + this._lastRefresh : Date.now()
		).toLocaleString();
	}
}

type Broadcaster = string; //`#${string}`;
// type Leaderboard = "channel" | "chatter";

class ChannelCache extends Cache<Broadcaster> {
	set channels(names: Iterable<Broadcaster>) {
		this._data = new Set(names);
	}
	get channels(): Array<Broadcaster> {
		return this._data.keys().toArray();
	}
}

const protocol = import.meta.env.DEV ? PUBLIC_DEVL_PROTO : PUBLIC_PROD_PROTO;
const apiUrl = import.meta.env.DEV ? PUBLIC_DEVL_API : PUBLIC_PROD_API;

export const channelCache = new ChannelCache(
	`${protocol}://${apiUrl}/channel/all`,
	"channels"
);
