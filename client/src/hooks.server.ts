import { sequence } from "@sveltejs/kit/hooks";
import {
	redirect,
	error,
	type Handle,
	type HandleServerError,
	type RequestEvent,
} from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";
import { getBaseURLFromRequest, isIpAddr, isLocalDomain } from "$lib/utils";
import { adminBucket, apiBucket } from "$lib/server/rate-limiter/token-bucket";
import { env } from "$env/dynamic/private";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

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
		null;
	if (!requestHost || isIpAddr(requestHost) || isLocalDomain(requestHost)) {
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
		event.locals.channel = null;
		return resolve(event);
	}

	if (await Rh.reroutable(event, requestedTenant)) {
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

	if (
		event.route.id === "/admin/login" &&
		event.cookies.get(ADMIN_SESSION_TOKEN) == null
	) {
		event.locals.logger.info({ route: event.route.id }, "checking rate limit");
		if (!adminBucket.consume(event.locals.client.cfconnecting, 1)) {
			error(429);
		}
	} else if (event.route.id?.startsWith("/api")) {
		event.locals.logger.info({ route: event.route.id }, "checking rate limit");
		if (!apiBucket.consume(event.locals.client.cfconnecting, 1)) {
			return new Response(
				JSON.stringify({ status: 429, error: "rate limit exceeded" }),
				{
					status: 429,
					headers: {
						"content-type": "application/json",
						"retry-after": (apiBucket.timeoutMs / 1000).toString(),
					},
				}
			);
		}
	}

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
