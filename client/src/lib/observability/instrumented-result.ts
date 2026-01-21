import { Result, Ok, Err } from "$lib/types/result";
import type { ApiError } from "$lib/types/error";
import { browser } from "$app/environment";

import { type Logger as ServerLogger, logger as serverLogger } from "./server/logger";
import { type ClientLogger, clientLogger } from "./client/logger";
import * as serverTracer from "./server/tracing";

let logger: ClientLogger | ServerLogger | any | null = null;
let tracer: typeof serverTracer | null = null;

async function getLogger() {
	if (browser && !logger) {
		logger = clientLogger;
	} else if (!logger) {
		logger = serverLogger;
	}

	return logger;
}

async function getTracing() {
	if (!browser && !tracer) {
		tracer = await import("./server/tracing");
	}

	return tracer;
}

export type InstrumentedResultOptions = {
	operation?: string;
	context?: Record<string, unknown>;
	logSuccess?: boolean;
	errorLevel?: "warn" | "error";
};

export async function instrumented<T, E = ApiError>(
	fn: () => Result<T, E> | Promise<Result<T, E>>,
	options: InstrumentedResultOptions = {}
): Promise<Result<T, E>> {
	const {
		operation = "unknown",
		context = {},
		logSuccess = false,
		errorLevel = "error"
	} = options;

	const startTime = performance.now();
	try {
		const result = await fn();
		const duration = performance.now() - startTime;

		if (result.isErr()) {
			await logError(
				result.variant.error as ApiError,
				{
					operation,
					duration,
					...context
				},
				errorLevel
			);
		} else if (logSuccess) {
			await logSuccess_({ operation, duration, ...context });
		}

		return result;
	} catch (err) {
		const duration = performance.now() - startTime;
		await logError(err as Error, { operation, duration, ...context }, "error");
		throw err;
	}
}

async function logError(
	error: ApiError | Error,
	context: Record<string, unknown>,
	level: "warn" | "error"
) {
	const logger = await getLogger();
	const tracing = await getTracing();

	const traceContext = tracing?.getTraceContext() ?? {};

	const logData = {
		...context,
		...traceContext,
		error:
			error instanceof Error
				? { message: error.message, stack: error.stack }
				: error
	};

	if (browser) {
		(logger as any)[level](
			logData,
			(error as any).message || "operation_failure"
		);
	} else {
		(logger as any)[level](
			logData,
			(error as any).message || "operation_failure"
		);
	}

	if (tracing && !browser) {
		tracing.recordException(error);
	}
}

async function logSuccess_(context: Record<string, unknown>) {
	const logger = await getLogger();
	const tracing = await getTracing();
	const traceContext = tracing?.getTraceContext() ?? {};

	if (browser) {
		(logger as any).debug({ ...context, ...traceContext }, "operation_success");
	} else {
		(logger as any).debug({ ...context, ...traceContext }, "operation_success");
	}
}

export class ObservableResult<T, E = ApiError> extends Result<T, E> {
	private constructor(
		variant: Result<T, E>["variant"],
		private readonly metadata: {
			operation?: string;
			context?: Record<string, unknown>;
		}
	) {
		super(variant);
	}

	static fromResult<T, E = ApiError>(
		result: Result<T, E>,
		metadata: { operation?: string; context?: Record<string, unknown> } = {}
	): ObservableResult<T, E> {
		return new ObservableResult(result.toEnum(), metadata);
	}

	static OkObserved<T, E = never>(
		value: T,
		metadata: { operation?: string; context?: Record<string, unknown> } = {}
	): ObservableResult<T, E> {
		return new ObservableResult({ _tag: "Ok", value }, metadata);
	}

	static ErrObserved<T = never, E = ApiError>(
		error: E,
		metadata: { operation?: string; context?: Record<string, unknown> } = {}
	): ObservableResult<T, E> {
		logError(
			error as ApiError,
			{ operation: metadata.operation, ...metadata.context },
			"error"
		);
		return new ObservableResult({ _tag: "Err", error }, metadata);
	}

	async logged(): Promise<this> {
		if (this.isErr()) {
			await logError(
				this.variant.error as ApiError,
				{ operation: this.metadata.operation, ...this.metadata.context },
				"error"
			);
		}

		return this;
	}

	override map<U>(fn: (value: T) => U): ObservableResult<U, E> {
		return ObservableResult.fromResult(super.map(fn), this.metadata);
	}

	override mapErr<F>(fn: (error: E) => F): ObservableResult<T, F> {
		return ObservableResult.fromResult(super.mapErr(fn), this.metadata);
	}

	override andThen<U>(fn: (value: T) => Result<U, E>): ObservableResult<U, E> {
		return ObservableResult.fromResult(super.andThen(fn), this.metadata);
	}
}
