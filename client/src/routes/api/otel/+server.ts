import { traceHandler as tracer } from "$lib/observability";
import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { type TelemetryPayload } from "$lib/observability";
import { logger } from "$lib/observability/server/logger.svelte";

// import type { DecoratedSpanAttributes } from "$lib/observability/server/tracing";

function isValidLogPayload(payload: TelemetryPayload): boolean {
	return payload.logs != null && Array.isArray(payload.logs);
}

// const buildAttributes = (attrs: DecoratedSpanAttributes) => {
//
// }

export const POST: RequestHandler = async (event: RequestEvent) => {
	return tracer.withAsyncSpan("process-client-logs", async (span) => {
		const { request } = event;
		try {
			const payload: TelemetryPayload = await request.json();
			if (!isValidLogPayload(payload)) {
				tracer.exception = {
					msg: "invalid log payload",
					span
				};

                span.end();

				return json({ error: "invalid payload" }, { status: 400 });
			}

			const attributes = {
				"client.addr": event.getClientAddress(),
				"client.id": payload.clientId,
				"client.session": payload.sessionId,
				"logs.count": payload.logs.length
			};

			span.setAttributes({ ...attributes });

			for (const entry of payload.logs) {
				const clientLogEntry = {
					span,
					entry,
					payload,
					addr: attributes["client.addr"]
				};
				logger[entry.level](
					{ clientLogEntry, message: entry.message },
					`"${entry.message}"`
				);
			}

            span.end();
			return json({ success: true, processed: payload.logs.length });
		} catch (err) {
			logger.error(
				{ error: err, timestamp: Date.now() },
				"unhandled error during client OTEL write"
			);
			tracer.exception = {
				msg:
					err instanceof Error
						? err
						: "unhandled error during client OTEL write" + `(ts=${Date.now()})`,
				span
			};

            span.end();
			return json(
				{
					error: "unhandled error during client OTEL write",
					timestamp: Date.now()
				},
				{ status: 500 }
			);
		}
	});
};
