import { Result } from "$lib/types";
import {
	trace,
	SpanStatusCode,
	type Span,
	type Tracer,
	type SpanContext
} from "@opentelemetry/api";
import { logger } from "./logger.svelte";
import { browser } from "$app/environment";

type TracedOptions = {
	name?: string;
	captureArgs?: boolean | number[];
	captureResult?: boolean;
	attributes?: Record<string, string | number | boolean>;
};

export type DecoratedSpanAttributes = {
	class: string;
	method: string;
	options: TracedOptions;
	args: Array<unknown>;
};

// helper function to build span attributes
export const createSpanAttributes = (
	attrs: DecoratedSpanAttributes
): Record<string, string | number | boolean> => {
	let attributes: Record<string, string | number | boolean> = {
		"code.namespace": attrs.class,
		"code.function": attrs.method,
		...(attrs.options.attributes || {})
	};

	if (attrs.options.captureArgs) {
		const indices =
			attrs.options.captureArgs === true
				? attrs.args.map((_, i) => i)
				: attrs.options.captureArgs;

		indices.forEach((i) => {
			if (i < attrs.args.length) {
				attributes[`code.function.arg${i}`] = safeSerialize(attrs.args[i]);
			}
		});
	}

	return attributes;
};

export class TraceHandler {
	private readonly globalTracerName: string;
	private readonly globalTracer: Tracer;

	_activeSpan: Span | undefined;
	_activeContext: SpanContext | undefined;

	constructor(globalTracerName = "sveltekit") {
		this.globalTracerName = globalTracerName;
		this.globalTracer = trace.getTracer(this.globalTracerName);
	}

	async withAsyncSpan<T extends unknown>(
		name: string,
		fn: (span: Span) => Promise<T>,
		options?: { attributes?: Record<string, string | number | boolean> }
	): Promise<T> {
		return this.tracer.startActiveSpan(name, async (span: Span) => {
			if (options?.attributes) {
				span.setAttributes(options.attributes);
			}

			const ctx = span.spanContext();
			return (await Result.fromPromise(fn(span))).match({
				Ok: (val) => {
					this.success = { span };
					span.end();
					logger.debug({ span }, `${ctx.traceId}.${ctx.spanId}: ASYNC OK`);

					return val;
				},
				Err: (err) => {
					this.exception = { span, msg: err };
					span.end();
					logger.error(
						{ error: err },
						`${ctx.traceId}.${ctx.spanId} ASYNC FAIL`
					);

					throw err;
				}
			});
		});
	}

	withSpan<T extends unknown>(
		name: string,
		fn: (span: Span) => T,
		options?: { attributes?: Record<string, string | number | boolean> }
	): T {
		return this.tracer.startActiveSpan(name, (span: Span) => {
			if (options?.attributes) {
				span.setAttributes(options.attributes);
			}

			const ctx = span.spanContext();
			return Result.from(fn, span).match({
				Ok: (val) => {
					this.success = { span };
					span.end();
					logger.debug({ span }, `${ctx.traceId}.${ctx.spanId}: SYNC OK`);

					return val;
				},
				Err: (err) => {
					this.exception = { span, msg: err };
					span.end();
					logger.error(
						{ error: err },
						`${ctx.traceId}.${ctx.spanId}: SYNC FAIL`
					);

					throw err;
				}
			});
		});
	}

	get activeSpan(): Span {
		if (!this._activeSpan) {
			this._activeSpan = trace.getActiveSpan();
		}

		return this._activeSpan!;
	}

	get context(): SpanContext {
		const span = this.activeSpan;

		return span.spanContext();
	}

	set exception({ msg, span }: { msg?: unknown; span: Span }) {
		const err = msg instanceof Error ? msg.message : String(msg);
		const handle = (s: Span) => {
			s.recordException(err);
			s.setStatus({ code: SpanStatusCode.ERROR, message: err });
		};

		if (span instanceof Promise) {
			span.then((s) => {
				handle(s);
			});
		} else {
			handle(span);
		}
	}

	set success({ msg, span }: { msg?: string; span: Span }) {
		const handle = (s: Span) => {
			s.setStatus({ code: SpanStatusCode.OK, message: msg });
		};

		if (span instanceof Promise) {
			span.then((s) => {
				handle(s);
			});
		} else {
			handle(span);
		}
	}

	get tracer(): Tracer {
		return this.globalTracer;
	}
}

export const traceHandler = new TraceHandler();

export function traced(
	options: TracedOptions = { captureResult: true, captureArgs: true }
) {
	function decorator<T, A extends unknown[], R>(
		method: (this: T, ...args: A) => Promise<R>,
		context: ClassMethodDecoratorContext<T, (this: T, ...args: A) => Promise<R>>
	): (this: T, ...args: A) => Promise<R>;

	function decorator<T, A extends unknown[], R>(
		method: (this: T, ...args: A) => R,
		context: ClassMethodDecoratorContext<T, (this: T, ...args: A) => R>
	): (this: T, ...args: A) => R;

	function decorator<T, A extends unknown[], R>(
		method: (this: T, ...args: A) => R,
		context: ClassMethodDecoratorContext<T, (this: T, ...args: A) => R>
	): (this: T, ...args: A) => R | Promise<R> {

		const methodName = String(context.name);
		const spanName = options.name || methodName;

		return function (this: T, ...args: A): R | Promise<R> {
			const className = (this as object)?.constructor?.name || "unknown";

			const attributes = createSpanAttributes({
				class: className,
				method: methodName,
				options,
				args
			});

			return traceHandler.withSpan(
				spanName,
				(span) => {
					const success = (result: R): R => {
						if (options.captureResult && result !== undefined) {
							if (span instanceof Promise) {
								span
									.then((s) =>
										s.setAttribute(
											"code.function.result",
											safeSerialize(result)
										)
									)
									.catch(logger.error);
							}
						}

						return result;
					};

					const except = (err: unknown): never => {
						throw err;
					};

					// this MIGHT throw, but we want to delegate error handling to the `with[?Async]Span`
					// method that we're wrapping (which handles span finalization automatically)
					const result = method.call(this, ...args);
					if (result instanceof Promise) {
						return result.then(success).catch(except);
					}

					return success(result as R);
				},
				{
					attributes
				}
			);
		};
	}

	return decorator;
}

function safeSerialize(value: unknown, maxLength = 1010): string {
	try {
		if (value === null || value === undefined) {
			return String(value);
		}
		if (typeof value === "string") {
			return value.length > maxLength
				? value.slice(0, maxLength) + "...(truncated)"
				: value;
		}
		if (typeof value === "number" || typeof value === "boolean") {
			return String(value);
		}

		const serialized = JSON.stringify(value);
		if (serialized.length > maxLength) {
			return serialized.slice(0, maxLength) + "...(truncated)";
		}

		return serialized;
	} catch (err) {
		return `[ERR_UNSERIALIZABLE]`;
	}
}
