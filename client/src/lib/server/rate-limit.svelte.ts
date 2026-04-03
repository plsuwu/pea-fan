import { logger as serverLogger } from "$lib/observability/server/logger.svelte";
import dayjs from "dayjs";

const RATE_LIMIT_BUCKETS = {
	router: "router",
	admin: "admin",
};

export type RateLimitBucket = keyof typeof RATE_LIMIT_BUCKETS;

export type Bucket = {
	tokens: number;
	lastRefill: number;
	timeoutExpiry?: number;
};

export type BucketConfig = {
	max: number;
	refillInterval: number;
	timeoutLength: number;
	timeoutBackoff: number;
	_tag: "admin" | "general" | "api";
};

export class TokenBucket<K> {
	private config: BucketConfig;
	private bucket = new Map<K, Bucket>();
	private logger: typeof serverLogger;

	constructor({
		max,
		refillInterval,
		timeoutLength,
		timeoutBackoff,
		_tag,
	}: BucketConfig) {
		this.config = {
			max,
			refillInterval,
			timeoutLength,
			timeoutBackoff: Math.max(timeoutBackoff, 1),
			_tag,
		};

		this.logger = serverLogger.child({
			limiter: this.config._tag,
			maxTokens: this.config.max,
			refillIntervalSeconds: this.config.refillInterval,
			timeoutLength: this.config.timeoutLength,
			timeoutBackoff: this.config.timeoutBackoff,
		});
	}

	public consume(key: K, cost = 1): boolean {
		const childLogger = this.logger.child({
			rateLimitKey: key,
			cost,
		});

		childLogger.info("[RATE_LIMITER] TRY_CONSUME");

		let client = this.bucket.get(key) ?? null;
		const now = Date.now();

		if (client === null) {
			client = this.new(key, cost, now);
			return true;
		}

		childLogger.setBindings({
			remainingTokens: client.tokens,
			refillTimer: new Date(client.lastRefill),
		});

		if (!this.checkTimeout(key, client, now)) {
			return false;
		}

		const refilledClient = this.refill(client, now);
		return this.decrement(key, refilledClient, cost);
	}

	public isTimedOutClient(key: K) {
		let client = this.bucket.get(key) ?? null;
		const now = Date.now();

		if (client === null) {
			this.new(key, 1, now);
			return false;
		}

		if (!this.checkTimeout(key, client!, now, false)) {
			return true;
		}

		return false;
	}

	public timeoutClient(key: K) {
		let client = this.bucket.get(key) ?? null;
		const now = Date.now();

		if (client === null) {
			this.new(key, 1, now);
			client = this.bucket.get(key)!;
		}

		client.tokens = 0;
		this.decrement(key, client, 1);
	}

	private checkTimeout(
		key: K,
		client: Bucket,
		ts: number,
		backoff = true
	): boolean {
		const childLogger = this.logger.child({
			key,
			clientExpiry: client.timeoutExpiry ?? "[NO_TIMEOUT_SET]",
			timestamp: new Date(ts),
		});

		childLogger.trace("[RATE_LIMITER]: CLIENT_TIMEOUT_CHECK");

		if (!client.timeoutExpiry) {
			childLogger.trace("[RATE_LIMITER]: CLIENT_NOT_RATE_LIMITED");
			return true;
		}

		if (client.timeoutExpiry && client.timeoutExpiry < ts) {
			childLogger.warn("[RATE_LIMITER]: REMOVE_CLIENT_TIMEOUT");
			client.timeoutExpiry = undefined;

			return true;
		} else {
			childLogger.warn("[RATE_LIMITER]: RATE_LIMITED_CLIENT");

			if (backoff) {
				const newExpiry =
					client.timeoutExpiry! +
					this.timeoutLengthMs * this.config.timeoutBackoff;

				childLogger.setBindings({
					previousTimeoutExpiry: dayjs(client.timeoutExpiry),
					adjustedTimeoutExpiry: newExpiry,
				});

				childLogger.info("[RATE_LIMITER]: ADJUST_TIMEOUT");
				client.timeoutExpiry = newExpiry;
			}

			return false;
		}
	}

	private decrement(key: K, client: Bucket, cost: number): boolean {
		const childLogger = this.logger.child({
			key,
			requestCost: cost,
			tokens: client.tokens,
		});

		if (client.tokens < cost) {
			childLogger.info("[RATE_LIMITER]: REQUEST_COST_EXCEEDS_CLIENT_TOKENS");

			this.bucket.set(key, {
				tokens: 0,
				lastRefill: Date.now(),
				timeoutExpiry: Date.now() + this.timeoutLengthMs,
			});
			return false;
		}

		client.tokens -= cost;
		this.bucket.set(key, client);

		childLogger.debug("[RATE_LIMITER]: REQUEST_ALLOWED");
		return true;
	}

	private new(key: K, cost: number, ts: number): Bucket {
		const client = {
			tokens: this.max - cost,
			lastRefill: ts,
			timedOut: false,
			timedOutAt: undefined,
		};

		this.bucket.set(key, client);

		const childLogger = this.logger.child({
			key,
			tokens: client.tokens,
		});
		childLogger.trace("[RATE_LIMITER] NEW_CLIENT_ADDED");
		return client;
	}

	private refill(client: Bucket, ts: number): Bucket {
		const childLogger = this.logger.child({
			tokens: client.tokens,
			lastRefill: client.lastRefill,
			timestamp: new Date(ts),
		});

		const amount = Math.floor((ts - client.lastRefill) / this.refillIntervalMs);
		childLogger.debug(
			{ adjustment: amount },
			"[RATE_LIMITER]: ADJUST_CLIENT_BUCKET"
		);

		client.tokens = Math.min(client.tokens + amount, this.max);
		client.lastRefill += amount * this.refillIntervalMs;

		return client;
	}

	get max(): number {
		return this.config.max;
	}

	get timeoutLengthMs(): number {
		return this.config.timeoutLength * 1000;
	}

	get refillIntervalMs(): number {
		return this.config.refillInterval * 1000;
	}
}

const ONE_HOUR = 60 * 60;
const HALF_HOUR = 60 * 30;
const FIVE_MINS = 60 * 5;

// bucket configured with 5 tokens; refills at a rate of 30 minutes/token
const adminRateLimiterConfig: BucketConfig = {
	max: 5,
	refillInterval: HALF_HOUR,
	timeoutLength: ONE_HOUR * 3,
	timeoutBackoff: 4,
	_tag: "admin",
};

const apiRateLimiterConfig: BucketConfig = {
	max: 10,
	refillInterval: 30,
	timeoutLength: FIVE_MINS,
	timeoutBackoff: 2,
	_tag: "api",
};

const routerRateLimiterConfig: BucketConfig = {
	max: 5,
	refillInterval: FIVE_MINS * 3,
	timeoutLength: ONE_HOUR * 6,
	timeoutBackoff: 2,
	_tag: "general",
};

export const apiRateLimiter = new TokenBucket<string>(apiRateLimiterConfig);
export const adminRateLimiter = new TokenBucket<string>(adminRateLimiterConfig);
export const routerRateLimiter = new TokenBucket<string>(
	routerRateLimiterConfig
);

export const tryConsumeToken = (
	key: string,
	bucket: RateLimitBucket = "router"
): boolean => {
	switch (bucket) {
		case "admin":
			return adminRateLimiter.consume(key);

		default:
			return routerRateLimiter.consume(key);
	}
};

export const shouldRateLimit = (key: string) => {
	return (
		adminRateLimiter.isTimedOutClient(key) ||
		routerRateLimiter.isTimedOutClient(key)
	);
};
