import { sequence } from "@sveltejs/kit/hooks";
import { redirect, type Handle } from "@sveltejs/kit";
import { URLS } from "$lib";
import { logger } from "$lib/observability/server/logger.svelte";
import { rtUtil } from "$lib/utils/routing";

const tenantHook: Handle = async ({ event, resolve }) => {
	// We don't explicitly create a new span on this function as we need to throw a type of
	// error - `redirect(..)` - if the tenant is invalid.
	//
	// SvelteKit handles this to perform the redirection, and our Span handler callback will
	// catch it, AT BEST logging an error instead of letting it propagate back up.
	const requestHost = event.request.headers.get("host") || null;
	const requestedTenant = requestHost?.split(".")[0] || null;

	const { api, base, proto } = URLS();

	if (
		!requestHost ||
		requestHost === `${proto}://${base}` ||
		requestHost === api ||
		!requestedTenant ||
		requestedTenant === base
	) {
		event.locals.logger.debug("no tenant requested");
		event.locals.channel = null;

		return resolve(event);
	}

	if (await rtUtil.reroutable(event, requestedTenant)) {
		event.locals.logger.debug(
			{ tenant: requestedTenant },
			"allowing route to valid tenant"
		);

		event.locals.channel = requestedTenant;
		return resolve(event);
	} else {
		event.locals.channel = null;
		event.locals.logger.warn(
			{ tenant: requestedTenant },
			"invalid tenant, redirecting to root URL"
		);

		const newUrl = `${proto}://${base}`;
		redirect(302, newUrl);
	}
};

const logInitHook: Handle = async ({ event, resolve }) => {
	event.locals.logger = logger.child({
		client: event.getClientAddress(),
		spanId: event.tracing.current.spanContext().spanId,
		traceId: event.tracing.current.spanContext().traceId
	});
	event.locals.logger.debug("pushed logger to request event locals");
	return resolve(event);
};

export const handle = sequence(logInitHook, tenantHook);
