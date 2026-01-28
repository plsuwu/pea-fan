import { URLS } from "$lib";
import { logger } from "./logger.svelte";
import { Result } from "$lib/types/result/result";
import { traced } from "./tracing";

const { api, proto } = URLS();

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
	}

	@traced()
	public async refresh(method = "GET"): Promise<void> {
		if (!this.ttlElapsed()) {
			logger.debug({ cache: this }, "cache up to date");
			return;
		}

		logger.debug({ cache: this }, "running cache refresh");
		const response = await fetch(this.url, { method });
		if (!response.ok) {
			logger.error({ response }, "error while updating a cache");

			return;
		}

		(await Result.fromPromise(response.json())).match({
			Ok: (data) => {
				this.data = data as Iterable<T>;
				this._lastRefresh = Date.now();
			},
			Err: (err) => {
				logger.error(
					{ cache: this, error: err },
					"error while reading response JSON"
				);

				throw err;
			}
		});

		logger.info({ cache: this }, "retrieved channels from endpoint");
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
			logger.warn("stale cache (refreshes on this request)");
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

type Broadcaster = `#${string}`;
// type Leaderboard = "channel" | "chatter";

class ChannelCache extends Cache<Broadcaster> {
	set channels(names: Iterable<Broadcaster>) {
		this._data = new Set(names);
	}
	get channels(): Array<Broadcaster> {
		return this._data.keys().toArray();
	}
}

export const channelCache = new ChannelCache(
	`${proto}://${api}/channel/irc-joins`,
	"channels"
);

