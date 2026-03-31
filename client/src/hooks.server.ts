import { sequence } from "@sveltejs/kit/hooks";
import {
	error,
	redirect,
	type Handle,
	type HandleServerError,
	type RequestEvent,
} from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";
import { getBaseURLFromRequest, isIpAddr, isLocalDomain } from "$lib/utils";
import { clientIsRateLimited } from "$lib/server/rate-limit.svelte";

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
	if (event.locals.rateLimited && event.url.pathname.includes("/admin")) {
		let response = await resolve(event);
		response = new Response(response.body, {
			...response,
			status: 429,
			headers: response.headers,
		});

		event.locals.logger.info(
			{ response },
			"[TENANCY]: DENY_ADMIN_RATE_LIMITED"
		);
		return response;
	}

	const requestHost = event.request.headers.get("host") || null;
	if (!requestHost || isIpAddr(requestHost) || isLocalDomain(requestHost)) {
		event.locals.logger.debug("[TENANCY]: ALLOW_ROUTE_DEFAULT");
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
		event.locals.logger.debug("[TENANCY]: ALLOW_ROUTE_NO_TENANT");
		event.locals.channel = null;

		return resolve(event);
	}

	if (await Rh.reroutable(event, requestedTenant)) {
		event.locals.logger.debug(
			{ tenant: requestedTenant },
			"[TENANCY]: ALLOW_ROUTE_VALID_TENANT"
		);

		event.locals.channel = requestedTenant;
		return resolve(event);
	} else {
		event.locals.channel = null;
		const baseFromRequest = getBaseURLFromRequest(requestHost);
		const redirection = `${Rh.proto}://${baseFromRequest}`;

		event.locals.logger.warn(
			{ tenant: requestedTenant, redirection },
			"[TENANCY]: DENY_ROUTE_INVALID_TENANT"
		);

		redirect(302, redirection);
	}
};

const logInitHook: Handle = async ({ event, resolve }) => {
	event.locals.client = getClientFromHeaders(event);
	event.locals.rateLimited = clientIsRateLimited(
		event.locals.client.cfconnecting
	);

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

	event.locals.logger.debug("[INIT_HOOK]: LOGGER_APPENDED");
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
