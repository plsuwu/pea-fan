import { buildHeaders } from "$lib/server/verify";
import type { RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { json } from "@sveltejs/kit";
import { randomUUID } from "node:crypto";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const HOOKS_ENDPOINT = `${Rh.apiAdmin}/helix/hooks`;

type HookInfo = {
	condition: {
		broadcaster_user_id: string;
	};
	id: string;
	type: string;
	status: "enabled" | "disabled";
	version: "1";
	cost: number;
	transport: {
		callback: string;
		method: string;
		secret: null;
	};
	created_at: string;
};

type BroadcasterHook = { data: [string, HookInfo][] };

// Hook retrieval handler
export const GET: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");

	try {
		let token = cookies.get(ADMIN_SESSION_TOKEN);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "GET",
			headers: buildHeaders(false, token),
		});

		const status = res.status;
		if (!res.ok) {
			const body = await res.json();

			logger.warn({ status, data: body }, "failed to complete action");
			return json({ status, data: body });
		}

		const body = await res.json();

		logger.debug({ status, data: body.data }, "action completed successfully");
		return json({ status, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};

// Hook deletion handler
export const DELETE: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");

	try {
		let token = cookies.get(ADMIN_SESSION_TOKEN);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "DELETE",
			headers: buildHeaders(false, token),
		});

		const status = res.status;
		const body = await res.json();

		if (!res.ok) {
			logger.warn({ status, error: body.err }, "failed to complete action");
			return json({ status: res.status, data: "action failed" });
		}

		logger.debug({ status: res.status }, "action completed successfully");
		return json({ status: res.status, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};

// Hook reset handler
export const PUT: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");
	try {
		const token = cookies.get(ADMIN_SESSION_TOKEN);

		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "PUT",
			headers: buildHeaders(false, token),
		});

		const body = await res.json();
		const status = res.status;
		if (!res.ok) {
			logger.warn({ status, error: body.err }, "failed to complete action");
			return json({ status: res.status, data: "action failed" });
		}

		logger.debug({ status, data: body.data }, "action completed successfully");
		return json({ status: body.status, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};
