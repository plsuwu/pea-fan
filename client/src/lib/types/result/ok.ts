import { Err } from "./err";
import type { IResult, Result, ResultPattern } from "./result";


export class Ok<T, E = never> implements IResult<T, E> {
	readonly _tag = "Ok" as const;

	constructor(public readonly value: T) {}

	isOk(): this is Ok<T, E> {
		return true;
	}

	isErr(): this is Err<T, E> {
		return false;
	}

	unwrap(): T {
		return this.value;
	}

	unwrapOr(_: T): T {
		return this.value;
	}

	unwrapOrElse(_: (err: E) => T): T {
		return this.value;
	}

	map<U>(fn: (val: T) => U): Result<U, E> {
		return new Ok(fn(this.value));
	}

	mapErr<F>(_: (error: E) => F) {
		return new Ok(this.value);
	}

	mapOr<U>(_: U, fn: (value: T) => U): U {
		return fn(this.value);
	}

	mapOrElse<U>(_: (error: E) => U, fn: (val: T) => U): U {
		return fn(this.value);
	}

	andThen<U>(fn: (value: T) => Result<U, E>): Result<U, E> {
		return fn(this.value);
	}

	orElse<F>(_: (err: E) => Result<T, F>): Result<T, F> {
		return new Ok(this.value);
	}

	match<R>(matcher: ResultPattern<T, E, R>): R {
		return matcher.Ok(this.value);
	}

	ok(): T | undefined {
		return this.value;
	}

	err(): E | undefined {
		return undefined;
	}

	toString(): string {
		return `Ok(${this.value})`;
	}
}
