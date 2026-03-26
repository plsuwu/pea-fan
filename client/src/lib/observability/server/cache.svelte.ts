import { logger } from "./logger.svelte";

const API_BASE_URL = "https://api.rat.moe";

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
		this._ttlMs = ttl;
		this._ttlMs = 1000;
	}

	public async refresh(method = "GET"): Promise<void> {
        logger.info("[CACHE] beginning cache update");
		if (!this.ttlElapsed()) {
			logger.debug({ cache: this }, "[CACHE] up to date");
			return;
		}

		logger.info({ cache: this }, "[CACHE] running refresh");
		const response = await fetch(this.url, { method });
		if (!response.ok) {
			logger.error({ response }, "[CACHE] error during update");
			return;
		}

		try {
			this.data = await response.json();
			this._lastRefresh = Date.now();

			logger.info({ cache: this }, "[CACHE] retrieved channel data");
		} catch (err) {
			logger.error({ error: err }, "[CACHE] channel cache update failed");
		}

		return;
	}

	public async exists(name: T): Promise<boolean> {
		await this.refresh();
		return this._data.has(name);
	}

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

type Broadcaster = string;

class ChannelCache extends Cache<Broadcaster> {
	set channels(names: Iterable<Broadcaster>) {
		this._data = new Set(names);
	}
	get channels(): Array<Broadcaster> {
		return this._data.keys().toArray();
	}
}


export const channelCache = new ChannelCache(
	`${API_BASE_URL}/channel/all`,
	"channels"
);
