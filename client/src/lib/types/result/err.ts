import type { Ok } from "./ok";
import type { IResult, Result, ResultPattern } from "./result";

export class Err<T, E> implements IResult<T, E> {
	readonly _tag = "Err" as const;

	constructor(public readonly error: E) {}

	isOk(): this is Ok<T, E> {
		return false;
	}

	isErr(): this is Err<T, E> {
		return true;
	}

	unwrap(): T {
		throw new Error(
			`Called unwrap on an Err: ${
				this.error instanceof Error ? this.error.message : this.error
			}`
		);
	}

	unwrapOr(val: T): T {
		return val;
	}

	unwrapOrElse(fn: (error: E) => T): T {
		return fn(this.error);
	}

	unwrapErr(): E {
		return this.error;
	}

	map<U>(_: (val: T) => U): Result<U, E> {
		return new Err(this.error);
	}

	mapErr<F>(fn: (error: E) => F): Result<T, F> {
		return new Err(fn(this.error));
	}

	mapOr<U>(def: U, _: (val: T) => U): U {
		return def;
	}

	mapOrElse<U>(fn: (error: E) => U, _: (value: T) => U): U {
		return fn(this.error);
	}

	andThen<U>(_: (value: T) => Result<U, E>): Result<U, E> {
		return new Err(this.error);
	}

	orElse<F>(fn: (error: E) => Result<T, F>): Result<T, F> {
		return fn(this.error);
	}

	match<R>(matcher: ResultPattern<T, E, R>): R {
		return matcher.Err(this.error);
	}

	ok(): T | undefined {
		return undefined;
	}

	err(): E | undefined {
		return this.error;
	}

	toString(): string {
		return `Err(${this.error})`;
	}
}
