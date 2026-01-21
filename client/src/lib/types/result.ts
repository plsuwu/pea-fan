import type { ApiError } from "./error";
import type { EnumType } from "./match";

type ResultVariants<T, E> = {
	Ok: { value: T };
	Err: { error: E };
};

export class Result<T, E = ApiError> {
	protected constructor(readonly variant: EnumType<ResultVariants<T, E>>) {}

	static Ok<T, E = never>(value: T): Result<T, E> {
		return new Result({ _tag: "Ok", value });
	}

	static Err<T = never, E = ApiError>(error: E): Result<T, E> {
		return new Result({ _tag: "Err", error });
	}

	isOk(): this is Result<T, never> & { variant: { _tag: "Ok"; value: T } } {
		return this.variant._tag === "Ok";
	}

	isErr(): this is Result<never, E> & { variant: { _tag: "Err"; error: E } } {
		return this.variant._tag === "Err";
	}

	match<R>(patterns: { Ok: (value: T) => R; Err: (error: E) => R }): R {
		if (this.variant._tag === "Ok") {
			return patterns.Ok(this.variant.value);
		}

		return patterns.Err(this.variant.error);
	}

	unwrap(): T {
		return this.match({
			Ok: (value) => value,
			Err: (error) => {
				throw new Error(`called unwrap on an Err: ${JSON.stringify(error)}`);
			}
		});
	}

	unwrapOr(fallback: T): T {
		return this.match({
			Ok: (value) => value,
			Err: () => fallback
		});
	}

	map<U>(fn: (value: T) => U): Result<U, E> {
		return this.match({
			Ok: (value) => Result.Ok(fn(value)),
			Err: (error) => Result.Err(error)
		});
	}

	mapErr<F>(fn: (error: E) => F): Result<T, F> {
		return this.match({
			Ok: (value) => Result.Ok(value),
			Err: (error) => Result.Err(fn(error))
		});
	}

	andThen<U>(fn: (value: T) => Result<U, E>): Result<U, E> {
		return this.match({
			Ok: fn,
			Err: (error) => Result.Err(error)
		});
	}

	orElse<F>(fn: (error: E) => Result<T, F>): Result<T, F> {
		return this.match({
			Ok: (value) => Result.Ok(value),
			Err: fn
		});
	}

	toEnum(): EnumType<ResultVariants<T, E>> {
		return this.variant;
	}

	static async fromPromise<T, E = Error>(
		promise: Promise<T>,
		mapErr?: (error: unknown) => E
	): Promise<Result<T, E>> {
		try {
			const val = await promise;
			return Result.Ok(val);
		} catch (error) {
			const mappedErr = mapErr ? mapErr(error) : (error as E);
			return Result.Err(mappedErr);
		}
	}
}

export const Ok = Result.Ok;
export const Err = Result.Err;
