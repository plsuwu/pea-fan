import { Ok, Err } from ".";

export interface ResultPattern<T, E, R> {
	Ok: (val: T) => R;
	Err: (err: E) => R;
}

export interface IResult<T, E> {
	readonly _tag: "Ok" | "Err";

	isOk(): this is Ok<T, E>;
	isErr(): this is Err<T, E>;

	unwrap(): T;
	unwrapOr(val: T): T;
	unwrapOrElse(fn: (err: E) => T): T;

	map<U>(fn: (value: T) => U): Result<U, E>;
	mapErr<F>(fn: (error: E) => F): Result<T, F>;
	mapOr<U>(def: U, fn: (val: T) => U): U;
	mapOrElse<U>(defFn: (error: E) => U, fn: (value: T) => U): U;
	andThen<U>(fn: (value: T) => Result<U, E>): Result<U, E>;
	orElse<F>(fn: (error: E) => Result<T, F>): Result<T, F>;

	match<R>(matcher: ResultPattern<T, E, R>): R;

	ok(): T | undefined;
	err(): E | undefined;
}

export type Result<T, E> = Ok<T, E> | Err<T, E>;

export const Result = {
	ok<T, E = never>(val: T): Result<T, E> {
		return new Ok(val);
	},

	err<T = never, E = unknown>(err: E): Result<T, E> {
		return new Err(err);
	},
    
	from<T>(fn: () => T): Result<T, Error> {
		try {
			return new Ok(fn());
		} catch (e) {
			return new Err(e instanceof Error ? e : new Error(String(e)));
		}
	},

	async fromPromise<T>(promise: Promise<T>): Promise<Result<T, Error>> {
		try {
			const val = await promise;
			return new Ok(val);
		} catch (e) {
			return new Err(e instanceof Error ? e : new Error(String(e)));
		}
	},

	isResult<T, E>(val: unknown): val is Result<T, E> {
		return val instanceof Ok || val instanceof Err;
	},

	all<T, E>(results: Array<Result<T, E>>): Result<Array<T>, E> {
		const vals: Array<T> = new Array();
		for (const res of results) {
			if (res.isErr()) {
				return new Err(res.error);
			}

			vals.push(res.value);
		}

		return new Ok(vals);
	},

	any<T, E>(results: Array<Result<T, E>>): Result<T, Array<E>> {
		const errors: Array<E> = new Array();
		for (const res of results) {
			if (res.isOk()) {
				return new Ok(res.value);
			}

			errors.push(res.error);
		}

		return new Err(errors);
	}
};
