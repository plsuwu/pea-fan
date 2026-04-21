import { logger } from "$lib/observability/server/logger.svelte";

interface Bucket {
	count: number;
	refilledMs: number;
	timeoutExp: number | null;
}

interface BucketCfg {
	max: number;
	refillSec: number;
	timeoutSec: number;
}

/**
 * Default token bucket configuration
 */
export const makeTokenBucketCfg = (
	max = 7,
	refillSec = 60,
	timeoutSec = 900 // 15 minutes
): BucketCfg => {
	return {
		max,
		refillSec,
		timeoutSec,
	};
};

export class TokenBucketRateLimit<K> {
	private storage = new Map<K, Bucket>();
	private _tag: string;
	private logger: typeof logger;

	public cfg: BucketCfg;

	constructor(tag: string, config = makeTokenBucketCfg()) {
		this.cfg = config;
		this._tag = tag;

		this.logger = logger.child({
			tag: this._tag,
		});
	}

	/**
	 * Use the client address to try consuming a token, timing the client out if they have
	 * exceeded the rate limit.
	 */
	public consume(key: K, cost: number): boolean {
		let bucket = this.storage.get(key) ?? null;
		if (this.isTimedOut(key, bucket)) {
			this.logger.setBindings({
				client: key,
				expiry: new Date(bucket!.timeoutExp!).toLocaleString(),
			});
			return false;
		}

		const now = Date.now();
		if (bucket === null) {
			bucket = {
				count: this.max - cost,
				refilledMs: now,
				timeoutExp: null,
			};
			this.logger.trace({ client: key }, "new rate limiter entry");
			this.storage.set(key, bucket);
			return true;
		}

		const refill = Math.floor((now - bucket.refilledMs) / this.refillMs);

		bucket.count = Math.min(bucket.count + refill, this.max);
		bucket.refilledMs = bucket.refilledMs + refill * this.refillMs;

		if (bucket.count < cost) {
			this.timeout(key, bucket);
			return false;
		}

		bucket.count -= cost;
		this.storage.set(key, bucket);

		return true;
	}

	/**
	 * Set a bucket's timeout timestamp to now and writes the updated bucket to the
	 * cache.
	 */
	private timeout(key: K, bucket: Bucket) {
		bucket.timeoutExp = Date.now() + this.timeoutMs;
		this.storage.set(key, bucket);
		this.logger.warn(
			{
				client: key,
				expiry: new Date(bucket.timeoutExp).toLocaleString(),
			},
			"set timeout on client"
		);
	}

	/**
	 * Checks whether a client is timed out, returning true if the client is timed out, or
	 * false otherwise.
	 *
	 * If the timeout has elapsed, this function also sets the timeout timestamp to `null`
	 * and writes the updated bucket to the cache prior to returning false.
	 */
	private isTimedOut(key: K, bucket: Bucket | null): boolean {
		if (bucket == null || bucket.timeoutExp == null) {
			this.logger.trace({ client: key }, "no bucket for client");
			return false;
		}

		const now = Date.now();
		const expiry = bucket.timeoutExp;

		if (expiry < now) {
			this.logger.debug(
				{ client: key, now, expiry },
				"client timeout has elapsed"
			);

			bucket.timeoutExp = null;
			this.storage.set(key, bucket);

			return false;
		}

		this.logger.trace(
			{ client: key, now, expiry },
			"client timeout not elapsed"
		);

		return true;
	}

	get max(): number {
		return this.cfg.max;
	}

	get refillMs(): number {
		return this.cfg.refillSec * 1000;
	}

	get timeoutMs(): number {
		return this.cfg.timeoutSec * 1000;
	}
}

export const apiBucket = new TokenBucketRateLimit<string>("api");

export const adminBucket = new TokenBucketRateLimit<string>("admin", {
	max: 3,
	refillSec: 60,
	timeoutSec: 3600,
});
