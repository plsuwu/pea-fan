import { logger } from "$lib/observability/server/logger.svelte";
import { env } from "$env/dynamic/public";
import { browser } from "$app/environment";
import clientLogger from "$lib/observability/client/logger";

const FIVE_MINUTES = 300_000;

export const INTL_API_BASE = env.PUBLIC_INTERNAL_API ?? "http://localhost:8080";
export const INTL_API_ROUTE = `${INTL_API_BASE}/api/v1`;

interface CacheMetadata {
	tag: string;
	logger: typeof logger;
	endpoint: string;
	ttl: number;
	lastRefresh: number | null;
}

export const util = {
	elapsed: (ttl: number, lastRefresh: number) => Date.now() > lastRefresh + ttl,
	endpoint: (suffix: string) => `${INTL_API_ROUTE}/${suffix}`,
};

export abstract class Cache<T> implements CacheMetadata {
	protected readonly fallback: T | T[];

	public readonly tag: string;
	public readonly endpoint: string;
	public readonly ttl: number;

	public logger: typeof logger;

	public data: Set<T> = new Set();
	public lastRefresh: number | null = null;

	constructor(
		tag: string,
		endpoint: string,
		fallback: T | T[],
		ttl: number = FIVE_MINUTES
	) {
		this.fallback = fallback;
		this.tag = tag;
		this.endpoint = endpoint;
		this.ttl = ttl;
		this.logger = logger.child({
			cacheTag: this.tag,
			cacheEndpoint: this.endpoint,
		});
	}

	abstract fetchData(): Promise<T | T[]>;
	abstract read(): Promise<T | T[]>;

	/**
	 * check for new data if TTL has expired.
	 *
	 * handles try/catch logic internally so that extending classes are able to defer
	 * handling errors if they don't wish to implement specific error handling logic
	 */
	public async refresh() {
        if (browser) {
            clientLogger.error("server-only request found in clientside request");
        }

		if (this.lastRefresh != null && !util.elapsed(this.ttl, this.lastRefresh)) {
			this.logger.trace(
				{
					lastRefresh: new Date(this.lastRefresh),
					nextRefresh: new Date(this.ttl + this.lastRefresh),
				},
				"not refreshing cache"
			);

			return;
		}

		// avoid refetch attempts on every request after a failure
		this.lastRefresh = Date.now();

		// try/catch to allow subclasses to defer errors to us.
		try {
			const newData = await this.fetchData();
			this.update(newData);

			this.logger.debug({ cachedData: this.data }, "refreshed cache");
		} catch (err) {
			this.logger.error(
				{ error: err },
				"failure while attempting to fetch cache update"
			);
            

			// this.update(this.fallback);
            // maybe just avoid updating if we outright fail
            return;
		}
	}

	public update(data: T | T[]): void {
		if (Array.isArray(data)) {
			this.data = new Set(data);
			return;
		}

		this.data = new Set([data]);
		return;
	}

	/**
	 * add a `T` or `T[]` to this cache
	 *
	 * Note that `Set` semantically overwrites duplicate values, but this method's
	 * purpose is to extend the cache rather than refresh.
	 */
	public add(data: T | T[]): void {
		if (Array.isArray(data)) {
			this.data = new Set([...this.data, ...data]);
			return;
		}

		this.data.add(data);
		return;
	}

	/**
	 * remove all elements from this cache
	 */
	public clear() {
		this.data.clear();
	}

	public async exists(val: T) {
		await this.refresh();
		return this.data.has(val);
	}
}
