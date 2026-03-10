import { sequence } from "@sveltejs/kit/hooks";
import { redirect, type Handle, type HandleServerError } from "@sveltejs/kit";
// import { URLS } from "$lib";
import { logger } from "$lib/observability/server/logger.svelte";
// import { rtUtil } from "$lib/utils/routing";
import { Rh } from "$lib/utils/route";
import { getBaseURLFromRequest, isIpAddr, isLocalDomain } from "$lib/utils";

// NOTE:
// most of the functions below don't explicitly create new spans as svelte's `redirect(..)` functionality
// throws an error under the hood if the tenant is invalid.
//
// sveltekit will internally handle this to perform the redirection so we don't want a span error handler
// attempting to catch it, even though it will apparently just log the error before propagating it back up to
// sveltekit's handlers - this seems like it is very likely to cause some evil bugs that are tricky to
// diagnose down the line...

export const handleError: HandleServerError = ({
	event,
	error,
	status,
	message,
}) => {
	const context = event.tracing.current
		? event.tracing.current.spanContext()
		: event.tracing.root.spanContext();

	const { spanId, traceId } = context;
	const { url, locals, getClientAddress } = event;

	logger.error(
		{
			error,
			extra: {
				url,
				locals,
				client: getClientAddress(),
				spanId,
				traceId,
			},
		},
		"ERROR"
	);

	let displayMessage;

	switch (status.toString()) {
		case "404":
			displayMessage = "the requested page doesn't exist";
			break;

		case "500":
			displayMessage = "an internal error occurred";
			break;

		default:
			displayMessage = "an unexpected error occurred";
			break;
	}

	return {
		message: displayMessage,
		code: status,
		traceId,
		spanId,
	};
};

const tenantHook: Handle = async ({ event, resolve }) => {
	const requestHost = event.request.headers.get("host") || null;
	if (!requestHost || isIpAddr(requestHost) || isLocalDomain(requestHost)) {
		event.locals.logger.debug(
			"not a tenant request (direct IP request or host missing)"
		);
		event.locals.channel = null;
		return resolve(event);
	}

	const requestedTenant = requestHost?.split(".")[0];
	// const { api, base, proto } = URLS();

	if (
		!requestedTenant ||
		/**
		 * API is on a dedicated port, so this might
		 * be redundant now
		 **/
		requestHost === Rh.api ||
		requestedTenant === Rh.base
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
		spanId: event.tracing.current.spanContext().spanId,
		traceId: event.tracing.current.spanContext().traceId,
	});
	event.locals.logger.debug("pushed logger to request event locals");
	return resolve(event);
};

export const handle = sequence(logInitHook, tenantHook);
