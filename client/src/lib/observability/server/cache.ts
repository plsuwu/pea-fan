import { URLS } from "$lib";
import { logger } from "./logger.svelte";
import { Err, Result } from "$lib/types";

const { api, proto } = URLS();

export abstract class Cache<T extends string> {
	readonly url: string;

	private _channels: Set<T> = new Set();
	private _ttlMs: number = 300000;
	private _lastRefresh: number | null = null;

	constructor(url: string, ttl = 300000) {
		this.url = url;

		this.refresh();
		this._ttlMs = ttl;
		this._lastRefresh = Date.now();
	}

	public async refresh<S extends Iterable<T> = Array<T>>(
		method = "GET"
	): Promise<void> {
		if (!this.ttlElapsed()) {
			logger.debug({ cached: this }, "cache up to date");
			return;
		}

		const response = await fetch(this.url, { method });
		if (!response.ok) {
			logger.error({ response }, "recv error response");
		}

		(await Result.fromPromise(response.json())).match({
			Ok: (channels) => {
				this.channels = channels as S;
			},
			Err: (err) => {
				logger.error(
					{ cached: this, error: err },
					"error while receiving response JSON"
				);
				return;
			}
		});

		logger.info({ cached: this }, "retrieved channels from endpoint");
	}

	public async exists(name: T): Promise<boolean> {
		await this.refresh();
		const contains = this._channels.has(name);
		return contains;
	}

	private ttlElapsed(): boolean {
		logger.info({ cached: this }, "checking ttlElapsed");
		return (
			this._lastRefresh == null || Date.now() > this._lastRefresh + this._ttlMs
		);
	}

	get apiEndpoint(): string {
		return this.url;
	}

	get channels(): Array<T> {
		return this._channels.keys().toArray();
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

	set channels(names: Iterable<T>) {
		this._channels = new Set(names);
	}

	set ttl(ttl: number) {
		this._ttlMs = ttl;
	}
}

type Broadcaster = `#${string}`;
class ChannelCache extends Cache<Broadcaster> {}

export const channelCache = new ChannelCache(
	`${proto}://${api}/channel/irc-joins`
);
