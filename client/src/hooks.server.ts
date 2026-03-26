import { sequence } from "@sveltejs/kit/hooks";
import { redirect, type Handle, type HandleServerError } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";
import { getBaseURLFromRequest, isIpAddr, isLocalDomain } from "$lib/utils";

// export const handleError: HandleServerError = ({ event, error, status }) => {
// 	const context = event.tracing?.current
// 		? event.tracing?.current.spanContext()
// 		: event.tracing?.root?.spanContext();
//
// 	const { spanId, traceId } = context;
// 	const { url, locals, getClientAddress } = event;
//
// 	logger.error(
// 		{
// 			error,
// 			extra: {
// 				url,
// 				locals,
// 				client: getClientAddress() ?? "undefined client IP",
// 				spanId,
// 				traceId,
// 			},
// 		},
// 		"ERROR"
// 	);
//
// 	let displayMessage;
//
// 	switch (status.toString()) {
// 		case "404":
// 			displayMessage = "the requested page doesn't exist";
// 			break;
//
// 		case "401":
// 			displayMessage = "you arent authorized to view this page";
// 			break;
//
// 		case "500":
// 			displayMessage = "an internal error occurred";
// 			break;
//
// 		default:
// 			displayMessage = "an unknown error occurred";
// 			break;
// 	}
//
// 	return {
// 		message: displayMessage,
// 		code: status,
// 		traceId,
// 		spanId,
// 	};
// };

const tenantHook: Handle = async ({ event, resolve }) => {
	const requestHost = event.request.headers.get("Host") || null;
	const xForwardedFor =
		event.request.headers.get("x-forwarded-for") || "NO_X-Forwarded-For";
	event.locals.logger.info(
		{
			clientAdapterIP: event.getClientAddress(),
			clientIp: xForwardedFor,
			eventUrl: event.url,
			request: event.request,
			route: event.route,
		},
		"[@ -> HOOK] REQUEST RX INITIAL"
	);
	if (!requestHost || isIpAddr(requestHost) || isLocalDomain(requestHost)) {
		event.locals.logger.debug(
			"not a tenant request (direct IP request or host missing)"
		);
		event.locals.channel = null;
		return resolve(event);
	}

	const requestedTenant = requestHost?.split(".")[0];
	if (
		!requestedTenant ||
		requestedTenant === Rh.base ||
		requestHost === Rh.base ||
		requestHost === Rh.apiBase
	) {
		event.locals.logger.debug("[ALLOW] (no tenant)");
		event.locals.channel = null;

		return resolve(event);
	}

	if (await Rh.reroutable(event, requestedTenant)) {
		event.locals.logger.debug(
			{ tenant: requestedTenant },
			"[ALLOW] (valid tenant)"
		);

		event.locals.channel = requestedTenant;
		return resolve(event);
	} else {
		event.locals.channel = null;
		const baseFromRequest = getBaseURLFromRequest(requestHost);
		const redirection = `${Rh.proto}://${baseFromRequest}`;

		event.locals.logger.warn(
			{ tenant: requestedTenant, redirection },
			"[REDIRECT] (invalid tenant)"
		);

		redirect(302, redirection);
	}
};

const logInitHook: Handle = async ({ event, resolve }) => {
	event.locals.logger = logger.child({
		client: event.getClientAddress(),
		spanId:
			event.tracing?.current?.spanContext().spanId ??
			event.tracing?.root?.spanContext()?.spanId ??
			"0",
		traceId:
			event.tracing?.current?.spanContext().traceId ??
			event.tracing?.root?.spanContext()?.traceId ??
			"0",
	});
	event.locals.logger.debug("pushed logger to request event locals");
	return resolve(event);
};

export const handle = sequence(logInitHook, tenantHook);
