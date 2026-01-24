import { trace, SpanStatusCode } from "@opentelemetry/api";
import { json, type RequestHandler } from "@sveltejs/kit";
import { type TelemetryPayload } from "$lib/observability";
import { logger } from "$lib/observability/server/logger.svelte";

function isValidLogPayload(payload: TelemetryPayload): boolean {
	return payload.logs != null && Array.isArray(payload.logs);
}

export const POST: RequestHandler = async ({ request }) => {
	const tracer = trace.getTracer("sveltekit");
	return tracer.startActiveSpan("process-client-logs", async (span) => {
		try {
			const payload: TelemetryPayload = await request.json();
			if (!isValidLogPayload(payload)) {
				span.setStatus({ code: 2, message: "invalid_payload" });
				return json({ error: "invalid_payload" }, { status: 400 });
			}

			span.setAttribute("client.id", payload.clientId);
			span.setAttribute("client.session_id", payload.sessionId);
			span.setAttribute("logs.count", payload.logs.length);

			for (const entry of payload.logs) {
				entry.context = { ...entry.context, ...span.spanContext() };
				const logData = {
					...entry.context,
					clientId: payload.clientId,
					sessionId: payload.sessionId,
					url: payload.url,
					clientTimestamp: entry.timestamp,
					source: "client"
				};

				console.log(logData);
				logger[entry.level](logData, entry.message);
			}

			span.setStatus({
				code: SpanStatusCode.OK,
				message: "processing_success"
			});

			span.end();
			return json({ success: true, processed: payload.logs.length });
		} catch (err) {
			span.recordException(err as Error);
			span.setStatus({
				code: SpanStatusCode.ERROR,
				message: "processing_failure"
			});

			span.end();
			logger.error({ error: err }, "failed to process client telemetry batch");
			return json({ error: "processing_failure" }, { status: 500 });
		}
	});
};
