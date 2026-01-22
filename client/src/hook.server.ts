import { logger } from "$lib/observability/server/logger";
import { getTraceContext } from "$lib/observability";
import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
	const { traceId, spanId } = getTraceContext();

	event.locals.traceId = traceId;
	event.locals.spanId = spanId;

	event.locals.logger = logger.child(
		{
			traceId,
			spanId,
			path: event.url.pathname,
			method: event.request.method
		},
		{ level: "info" }
	);

	event.locals.logger.info("request_init");

	try {
		const response = await resolve(event);
		event.locals.logger.info({ status: response.status }, "request_complete");

		return response;
	} catch (err) {
		event.locals.logger.error({ error: err }, "request_failure");

		throw err;
	}
};
