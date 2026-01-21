import { logger, getTraceContext } from "$lib/observability";
import type { Handle, HandleServerError } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
	const startTime = performance.now();
	const { traceId, spanId } = getTraceContext();

	event.locals.traceId = traceId;
	event.locals.spanId = spanId;

	event.locals.logger = logger.child({
		traceId,
		spanId,
		path: event.url.pathname,
		method: event.request.method
	});

	event.locals.logger.info("request_init");

	try {
		const response = await resolve(event);
		const duration = performance.now() - startTime;
		event.locals.logger.info(
			{ duration, status: response.status },
			"request_complete"
		);

		return response;
	} catch (err) {
		const duration = performance.now() - startTime;
		event.locals.logger.error({ duration, error: err }, "request_failure");

		throw err;
	}
};

export const handleError: HandleServerError = async ({
	error,
	event,
	status,
	message
}) => {
	const { traceId } = getTraceContext();

	logger.error(
		{
			error,
			path: event.url.pathname,
			status,
			traceId
		},
		message || "unhandled server error"
	);

	return {
		message: "an unexpected error has occurred",
		traceId
	};
};
