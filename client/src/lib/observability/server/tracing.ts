import type { ApiError } from "$lib/observability/types";
import { TRACER_NAME } from "$lib/types";
import {
	trace,
	SpanStatusCode,
	type Span,
	type Tracer
} from "@opentelemetry/api";

export function getTracer(): Tracer {
	return trace.getTracer(TRACER_NAME);
}

export function getCurrentSpan(): Span | undefined {
	return trace.getActiveSpan();
}

export function getTraceContext(): { traceId?: string; spanId?: string } {
	const span = getCurrentSpan();
	if (!span) return {};

	const spanContext = span.spanContext();
	return {
		traceId: spanContext.traceId,
		spanId: spanContext.spanId
	};
}

export function recordException(error: Error | ApiError, span?: Span): void {
	const activeSpan = span || getCurrentSpan();
	if (!activeSpan) return;

	if (error instanceof Error) {
		activeSpan.recordException(error);
	} else {
		activeSpan.recordException(new Error(error.message));
		activeSpan.setAttributes({
			"error.code": String(error.code),
			"error.details": JSON.stringify(error.details)
		});
	}

	activeSpan.setStatus({ code: SpanStatusCode.ERROR, message: error.message });
}

export async function withSpan<T>(
	name: string,
	fn: (span: Span) => Promise<T>,
	options?: { attributes?: Record<string, string | number | boolean> }
): Promise<T> {
	const tracer = getTracer();

	return tracer.startActiveSpan(name, async (span) => {
		if (options?.attributes) {
			span.setAttributes(options.attributes);
		}

		try {
			const result = await fn(span);
			span.setStatus({ code: SpanStatusCode.OK });
			return result;
		} catch (error) {
			recordException(error as Error, span);
			throw error;
		} finally {
			span.end();
		}
	});
}

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
