interface Bucket {
	count: number;
	refilledMs: number;
}

export class TokenBucketRateLimit<K> {
	private storage = new Map<K, Bucket>();

	public max: number;
	public refillSec: number;

	constructor(max: number, refillSec: number) {
		this.max = max;
		this.refillSec = refillSec;
	}

	public consume(key: K, cost: number): boolean {
		let bucket = this.storage.get(key) ?? null;
		const now = Date.now();

		if (bucket === null) {
			bucket = {
				count: this.max - cost,
				refilledMs: now,
			};

			this.storage.set(key, bucket);
			return true;
		}

		const refill = Math.floor(
			(now - bucket.refilledMs) / (this.refillSec * 1000)
		);

		bucket.count = Math.min(bucket.count + refill, this.max);
		bucket.refilledMs = bucket.refilledMs + refill * this.refillSec * 1000;

		if (bucket.count < cost) {
			this.storage.set(key, bucket);
			return false;
		}

		bucket.count -= cost;
		this.storage.set(key, bucket);

		return true;
	}
}

export const apiBucket = new TokenBucketRateLimit<string>(5, 60);
