import { sequence } from "@sveltejs/kit/hooks";
import {
	redirect,
	type Handle,
	type HandleServerError,
	type RequestEvent,
} from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";
import { getBaseURLFromRequest, isIpAddr, isLocalDomain } from "$lib/utils";

export const handleError: HandleServerError = ({ event, error, status }) => {
	const context = event.tracing?.current
		? event.tracing?.current.spanContext()
		: event.tracing?.root?.spanContext();

	const { spanId, traceId } = context;
	const { url, locals } = event;

	logger.error(
		{
			error,
			extra: {
				url,
				locals,
				spanId,
				traceId,
			},
		},
		"ERROR"
	);

	let displayMessage;

	switch (status) {
		case 401:
			displayMessage = "you arent authorized to view this page";
			break;

		case 404:
			displayMessage = "the requested page doesn't exist";
			break;

		case 429:
			displayMessage = "too many requests";
			break;

		case 500:
			displayMessage = "an internal error occurred";
			break;

		default:
			displayMessage = "an unknown error occurred";
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
	const requestHost =
		event.request.headers.get("host") ??
		event.request.headers.get("x-host") ??
		null;
	if (!requestHost || isIpAddr(requestHost) || isLocalDomain(requestHost)) {
		// event.locals.logger.trace("allowing by default");
		event.locals.channel = null;

		return resolve(event);
	}

	const requestedTenant = requestHost?.split(".")[0];
	if (
		!requestedTenant ||
		requestedTenant === Rh.base ||
		requestHost === Rh.base ||
		requestHost === Rh.apiv1
	) {
		// event.locals.logger.trace("allowing base request");
		event.locals.channel = null;

		return resolve(event);
	}

	if (await Rh.reroutable(event, requestedTenant)) {
		// event.locals.logger.trace(
		// 	{ tenant: requestedTenant },
		// 	"allowing route to valid tenant"
		// );

		event.locals.channel = requestedTenant;
		return resolve(event);
	} else {
		event.locals.logger.warn(
			{ tenant: requestedTenant },
			"denying route to invalid tenant"
		);
		event.locals.channel = null;
		const baseFromRequest = getBaseURLFromRequest(requestHost);
		const redirection = `${Rh.proto}://${baseFromRequest}`;

		redirect(302, redirection);
	}
};

const logInitHook: Handle = async ({ event, resolve }) => {
	event.locals.client = getClientFromHeaders(event);
	event.locals.logger = logger.child({
		client: event.locals.client,
		route: event.url.href,
		rateLimited: event.locals.rateLimited,
		spanId:
			event.tracing?.current?.spanContext().spanId ??
			event.tracing?.root?.spanContext()?.spanId ??
			"0",
		traceId:
			event.tracing?.current?.spanContext().traceId ??
			event.tracing?.root?.spanContext()?.traceId ??
			"0",
	});

	event.locals.logger.info(
		{
			request: {
				body: event.request.body,
				method: event.request.method,
				destination: event.request.destination,
				headers: Object.fromEntries(event.request.headers),
			},
		},
		"logger init"
	);

	// // TODO:  check if we should rate limit
	// if (shouldRateLimit(event.locals.client.cfconnecting)) {
	// 	let response = await resolve(event);
	// 	response = new Response(response.body, {
	// 		...response,
	// 		status: 429,
	// 		headers: response.headers,
	// 	});
	//
	// 	event.locals.logger.warn(
	// 		{ response },
	// 		"[TENANCY]: denying rate limited client"
	// 	);
	// 	return response;
	// }

	return resolve(event);
};

export const handle = sequence(logInitHook, tenantHook);

function getClientFromHeaders(event: RequestEvent) {
	const headers = event.request.headers;

	const userAgent = headers.get("user-agent") ?? "MISSING";
	const cfconnecting =
		headers.get("cf-connecting-ip") ?? event.getClientAddress();
	const xforwardedList = headers
		.get("x-forwarded-for")
		?.split(",")
		.map((ip) => ip.trim()) ?? ["MISSING"];

	return {
		userAgent,
		cfconnecting,
		xforwarded: xforwardedList[xforwardedList.length - 1],
		xforwardedList,
	};
}
