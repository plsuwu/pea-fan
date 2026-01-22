import { json, type RequestHandler } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger";
import type { TelemetryPayload } from "$lib/observability";

function isValidLogPayload(payload: TelemetryPayload): boolean {
	return payload.logs != null && Array.isArray(payload.logs);
}

export const POST: RequestHandler = async ({ request }) => {
	try {
		const payload: TelemetryPayload = await request.json();
		if (!isValidLogPayload(payload)) {
			return json({ error: "invalid_payload" }, { status: 400 });
		}

		for (const entry of payload.logs) {
			const logData = {
				...entry.context,
				clientId: payload.clientId,
				sessionId: payload.sessionId,
				url: payload.url,
				clientTimestamp: entry.timestamp,
				source: "client"
			};

			const level = entry.level as keyof typeof logger;
			if (typeof logger[level] === "function") {
				(logger as any)[level](logData, entry.message);
			} else {
				logger.debug(logData, entry.message);
			}
		}

		return json({ success: true, processed: payload.logs.length });
	} catch (err) {
		logger.error({ error: err }, "failed to process client telemetry batch");
		return json({ error: "processing_failure" }, { status: 500 });
	}
};
