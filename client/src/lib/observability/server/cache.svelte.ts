import { logger } from "./logger.svelte";
import { PUBLIC_API_BASE_URL } from "$env/static/public";

const API_BASE_URL = `${PUBLIC_API_BASE_URL}/api/v1`;

export type RawIrcInfo = {
	likely_missing: string[];
	full_list: string[];
	current_joins: string[];
};

export abstract class Cache<T> {
	readonly url: string;
	readonly name: string;

	protected logger: typeof logger;

	protected _data: Set<T> = new Set();
	protected _lastRefresh: number | null = null;
	protected _ttlMs: number = 300000; // 300 secs = 5 mins

	constructor(url: string, cacheName: string, ttl = 300000) {
		this.url = url;
		this.name = cacheName;
		this._ttlMs = ttl;

		this.logger = logger.child({
			cacheFor: this.url,
			cacheName: this.name,
			ttlMs: this._ttlMs,
		});
	}

	public async refresh(method = "GET"): Promise<void> {
		// this.logger.debug("running cache update check");
		if (!this.ttlElapsed()) {
			// this.logger.debug("up to date");
			return;
		}

		// this.logger.info("beginning cache refresh");
		try {
			const res = await fetch(this.url, { method });
			if (!res.ok) {
				this.logger.error({ response: res }, "error during cache refresh");
				this.data = new Array();
				return;
			}

			this._lastRefresh = Date.now();

			const body = await res.json();
			this.data = body.data;

			// this.logger.info({ cache: this }, "refreshed cache data");
		} catch (err) {
			this.logger.error({ error: err }, "failed to update cache");
			this.data = new Array();
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
			this.logger.warn("stale data in cache, attempting refresh");
		}

		return stale;
	}

	// setters

	set data(names: Iterable<T>) {
		this._data.clear();
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

class AnnouncementCache {
	public logger: typeof logger;

	public url: string;
	public name: string;
	public key: string = "_";

	public data: Map<string, { content: string | null; hash: string | null }> =
		new Map();
	public lastRefresh: number | null = null;
	public ttlMs: number = 300_000; // = 300 secs = 5 mins

	constructor(url: string, name: string) {
		this.url = url;
		this.name = name;

		this.data.set(this.key, {
			content: null,
			hash: null,
		});

		this.logger = logger.child({
			cacheName: this.name,
			cacheUrl: this.url,
			mapKey: this.key,
		});
	}

	public async getAnnouncement(): Promise<{
		content: string | null;
		hash: string | null;
	}> {
		await this.refresh();
		return this.data.get(this.key) ?? { content: null, hash: null };
	}

	public async refresh(): Promise<void> {
		if (this.ttlElapsed()) {
			await this.fetchFromApi();
		}

		return;
	}

	public async fetchFromApi() {
		// updated this regardless of if fetch was successful; if this fails here i assume
		// it PROBABLY won't help to call it again on the next page load
		this.lastRefresh = Date.now();

		// will set this up to fetch from postgres but i'll do this later...

		const data = { content: null, hash: null };
		this.data.set(this.key, { content: data.content, hash: data.hash });
	}

	protected ttlElapsed(): boolean {
		const stale =
			this.lastRefresh == null || this.lastRefresh + this.ttlMs < Date.now();
		if (stale) {
			this.logger.warn("stale data in cache, attempting refresh");
		}

		return stale;
	}
}

export const announcementCache = new AnnouncementCache(
	`${API_BASE_URL}/announcements`,
	"announcements"
);
