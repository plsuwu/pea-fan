import { logger } from "$lib/observability/server/logger.svelte";
import { createError } from "$lib/observability";
import { redirect, type Handle, type RequestEvent } from "@sveltejs/kit";
import { sequence } from "@sveltejs/kit/hooks";
import { URLS } from "$lib";
import type { Logger } from "pino";
import { channelCache } from "$lib/observability/server/cache";
import { Result } from "$lib/types";

async function reroutable(
	event: RequestEvent,
	channel: string,
	apiUrl: string,
	childLogger: Logger
) {
	const isValid = await channelCache.exists(`#${channel}`);
	event.locals.logger.info({ isValid });

	// const channelEndpoint = `${apiUrl}/channel/irc-joins`;
	// try {
	// 	const channelReq = await event.fetch(channelEndpoint, {
	// 		method: "GET"
	// 	});
	//
	// 	if (!channelReq.ok) {
	// 		const deserialized = {
	// 			status: channelReq.status,
	// 			statusText: channelReq.statusText,
	// 			url: channelReq.url,
	// 			type: channelReq.type,
	// 			headers: Object.fromEntries(channelReq.headers)
	// 		};
	//
	// 		createError(channelReq.status, "error status code on channel list fetch")
	// 			.withTraceContext()
	// 			.details({ ...deserialized })
	// 			.buildAndLog();
	//
	// 		return false;
	// 	}
	//
	// 	const body: Array<string> = await channelReq.json();
	// 	childLogger.setBindings({
	// 		...childLogger.bindings(),
	// 		body: [...body.slice(0, 10), `...(${body.length - 10} more)`],
	// 		body_length: body.length
	// 	});
	//
	// 	childLogger.debug("retrieved viable tenant names");
	//
	// 	return body.includes(`#${requestChannel}`);
	// } catch (err) {
	// 	if (err instanceof Error) {
	// 		createError(err.name, err.message)
	// 			.withTraceContext()
	// 			.details({ cause: err.cause, stack: err.stack })
	// 			.buildAndLog();
	// 	} else {
	// 		const e = JSON.stringify((err as any).toString());
	// 		childLogger.error(e);
	// 	}
	//
	return true;
	// }
}

const tenantHook: Handle = async ({ event, resolve }) => {
	const requestHost = event.request.headers.get("host") || null;
	const requestSubdomain = requestHost?.split(".")[0] || null;

	const { api, base, proto } = URLS();

	if (
		!requestHost ||
		requestHost === `${proto}://${base}` ||
		requestHost === api ||
		!requestSubdomain ||
		requestSubdomain === base
	) {
		event.locals.logger.debug("no tenant found for request");
		event.locals.channel = null;
		return resolve(event);
	}

	const childLogger = event.locals.logger.child({
		tenant: requestSubdomain
	});

	if (
		await reroutable(event, requestSubdomain, `${proto}://${api}`, childLogger)
	) {
		childLogger.debug("routing to valid tenant");
		event.locals.channel = requestSubdomain;
		return resolve(event);
	} else {
		childLogger.warn("unable to route to specified tenant");
		event.locals.channel = null;
		const newUrl = `${proto}://${base}${event.url.pathname}${event.url.search}`;

		redirect(302, newUrl);
	}
};

const logInitHook: Handle = async ({ event, resolve }) => {
	event.locals.logger = logger.child({
		context: {
			...event.tracing.current.spanContext(),
			url: event.url,
			client: event.getClientAddress()
		}
	});
	event.locals.logger.debug("pushed logger to request event locals");
	return resolve(event);
};

export const handle = sequence(logInitHook, tenantHook);
