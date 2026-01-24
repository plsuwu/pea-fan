import { Result } from "$lib/types";
import {
	trace,
	SpanStatusCode,
	type Span,
	type Tracer,
	type SpanContext
} from "@opentelemetry/api";
import { logger } from "./logger.svelte";

export class TraceHandler {
	private readonly globalTracerName: string;
	private readonly globalTracer: Tracer;

	_activeSpan: Span | undefined;
	_activeContext: SpanContext | undefined;

	constructor(globalTracerName = "client-tracer") {
		this.globalTracerName = globalTracerName;
		this.globalTracer = trace.getTracer(this.globalTracerName);
	}

	async withSpan<T>(
		name: string,
		fn: (span: Span) => Promise<T>,
		options?: { attributes?: Record<string, string | number | boolean> }
	): Promise<T> {
		return this.tracer.startActiveSpan(name, async (span: Span) => {
			if (options?.attributes) {
				span.setAttributes(options.attributes);
			}

			return (await Result.fromPromise(fn(span))).match({
				Ok: (val: T) => {
					this.success = span;
					logger.info({ span }, "span ok");
					span.end();

					return val;
				},
				Err: (err: Error) => {
					this.exception = { err, currSpan: span };
					logger.error({ error: err }, "span fail");
					span.end();

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

	set exception({ err, currSpan }: { err: Error; currSpan?: Span }) {
		const span = currSpan || this.activeSpan;
		if (!span) {
			return;
		}

		span.recordException(err);
		span.setStatus({ code: SpanStatusCode.ERROR, message: err.message });
	}

	set success(span: Span) {
		span.setStatus({ code: SpanStatusCode.OK });
	}

	get tracer(): Tracer {
		return this.globalTracer;
	}
}

export const traceHandler = new TraceHandler();

// export function recordException(error: Error | ApiError, span?: Span): void {
// 	const activeSpan = span || getCurrentSpan();
// 	if (!activeSpan) return;
//
// 	if (error instanceof Error) {
// 		activeSpan.recordException(error);
// 	} else {
// 		activeSpan.recordException(new Error(error.message));
// 		activeSpan.setAttributes({
// 			"error.code": String(error.code),
// 			"error.details": JSON.stringify(error.details)
// 		});
// 	}
//
// 	activeSpan.setStatus({ code: SpanStatusCode.ERROR, message: error.message });
// }
//
// export async function withSpan<T>(
// 	name: string,
// 	fn: (span: Span) => Promise<T>,
// 	options?: { attributes?: Record<string, string | number | boolean> }
// ): Promise<T> {
// 	const tracer = getTracer();
//
// 	return tracer.startActiveSpan(name, async (span) => {
// 		if (options?.attributes) {
// 			span.setAttributes(options.attributes);
// 		}
//
// 		try {
// 			const result = await fn(span);
// 			span.setStatus({ code: SpanStatusCode.OK });
// 			return result;
// 		} catch (error) {
// 			recordException(error as Error, span);
// 			throw error;
// 		} finally {
// 			span.end();
// 		}
// 	});
// }
//
// export function traced(spanName?: string) {
// 	return function <T extends (...args: Array<any>) => Promise<any>>(
// 		target: any,
// 		propertyKey: string,
// 		descriptor: TypedPropertyDescriptor<T>
// 	) {
// 		const originalMethod = descriptor.value!;
// 		descriptor.value = async function (...args: Array<any>) {
// 			return withSpan(
// 				spanName || `${target.constructor.name}.${propertyKey}`,
// 				() => originalMethod.apply(this, args)
// 			);
// 		} as T;
//
// 		return descriptor;
// 	};
// }
//
// export function getCurrentSpan(): Span | undefined {
// 	return trace.getActiveSpan();
// }
//
// export function getTraceContext(): { traceId?: string; spanId?: string } {
// 	const span = getCurrentSpan();
// 	if (!span) return {};
//
// 	const spanContext = span.spanContext();
// 	return {
// 		traceId: spanContext.traceId,
// 		spanId: spanContext.spanId
// 	};
// }
