import { sequence } from "@sveltejs/kit/hooks";
import {
	redirect,
	type Handle,
	type HandleServerError,
	type RequestEvent,
} from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { routeManager } from "$lib/utils/route";

// TODO fix this bullshit ohg ymtifuhfkjfhljk
export const handleError: HandleServerError = ({ status }) => {
	let displayMessage;
	switch (status) {
		case 401:
			displayMessage = "you arent authorized to view this page";
			break;

		case 404:
			displayMessage = "the requested page doesn't exist";
			break;

		case 429:
			displayMessage = "rate limit exceeded";
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
	};
};

const tenantHook: Handle = async ({ event, resolve }) => {

	const requestHost =
		event.request.headers.get("host") ??
		event.request.headers.get("x-host") ??
		"missing_host";

	const urlParts = requestHost?.split(".");

	if (
		urlParts.length < 3 ||
		requestHost === routeManager.host ||
		requestHost === routeManager.api.external ||
		requestHost === routeManager.deriveBase(requestHost)
	) {
		event.locals.channel = null;
		return resolve(event);
	}

	const requestedTenant = urlParts[0];
	if (await routeManager.reroutable(requestedTenant)) {
		event.locals.channel = requestedTenant;
		return resolve(event);
	} else {
		const redirection = `${routeManager.proto}://${routeManager.host}`;
		event.locals.logger.warn(
			{ tenant: requestedTenant, redirection },
			"denying route - redirecting"
		);

		event.locals.channel = null;
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
