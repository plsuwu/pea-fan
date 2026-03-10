import { ADMIN_COOKIE_KEY, INTERNAL_POST_TOKEN_RAW } from "$env/static/private";
import type { PageServerLoad } from "./$types";
import type { Actions, Cookies, RequestEvent } from "@sveltejs/kit";
import { redirect } from "@sveltejs/kit";
import { logger } from "$lib/observability/server/logger.svelte";
import { Rh } from "$lib/utils/route";

// TODO:
// -------------------------------------------------------------------
// - endpoint needs rate limit hook,
// - HMAC-based message signing/verification,
// - also probably perform verification (or part of the verification)
//    in a server hook instead of here.

function getClientInfo(request: Request, getClientAddress: () => string) {
	const userAgent = request.headers.get("user-agent") ?? "[NO_USER_AGENT]";
	const host = request.headers.get("host") ?? "[NO_HOST]";
	const cookie = request.headers.get("cookie") ?? "[NO_COOKIE]";
	return {
		client: {
			addr: getClientAddress(),
			url: request.url,
			headers: {
				user_agent: userAgent,
				host,
				cookies: cookie,
			},
		},
	};
}

export const load: PageServerLoad = async (event: RequestEvent) => {
	const { fetch, cookies, request, getClientAddress } = event;
	const token = (cookies.get(ADMIN_COOKIE_KEY) as string) ?? "AAAAA";

	const res = await fetch("/api/verify-token", {
		method: "POST",
		body: JSON.stringify({ token }),
	});

	const { verified } = await res.json();
	logger.info({ verified });

	if (!token || (token && verified !== true)) {
		logger.warn(
			{ ...getClientInfo(request, getClientAddress) },
			"[ADMIN] unauthorized access attempt"
		);

		cookies.set(ADMIN_COOKIE_KEY, "", {
			path: "/",
			expires: new Date(0),
		});

		redirect(302, "/admin/login");
	} else {
		cookies.set(ADMIN_COOKIE_KEY, token, {
			path: "/",
		});

		return;
	}
};

async function verifyTokenOnAction(
	cookies: Cookies,
	request: Request,
	getClientAddress: () => string,
	fetch: typeof globalThis.fetch
) {
	const token = (cookies.get(ADMIN_COOKIE_KEY) as string) ?? "****************";
	const res = await fetch("/api/verify-token", {
		method: "POST",
		body: JSON.stringify({ token }),
	});

	const { verified } = await res.json();
	if (!verified) {
		logger.warn(
			{ ...getClientInfo(request, getClientAddress) },
			"[ACTION_ADMIN] unauthorized access attempt"
		);

		cookies.set(ADMIN_COOKIE_KEY, "", {
			path: "/",
			expires: new Date(0),
		});

		return null;
	}

	return token;
}

// TODO:
//  implement serverside token hashing and replace this
function buildAuthHeaders(
	{ isJSON }: { isJSON?: boolean } = { isJSON: false }
) {
	const headers = new Headers();
	headers.set("authorization", INTERNAL_POST_TOKEN_RAW);

	if (isJSON) {
		headers.set("content-type", "application/json");
	}

	return headers;
}

export const actions = {
	update: async ({ cookies, request, fetch, getClientAddress }) => {
		const verifiedToken = await verifyTokenOnAction(
			cookies,
			request,
			getClientAddress,
			fetch
		);
		if (!verifiedToken) {
			redirect(302, "/admin/login");
		}

		const formData = await request.formData();
		const type = formData.get("type") as string as "channel" | "chatter";

		const current = formData.get("current") as string;
		const historic = formData.get("historic") as string;

		if (!type || !current || !historic) {
			return {
				success: false,
				message: "missing one of 'type'/'current'/'historic'",
			};
		}

		const headers = buildAuthHeaders({ isJSON: true });
		const data = { current, historic: JSON.parse(historic) };

		const { success, status, body } = await runUpdate(
			type,
			data,
			fetch,
			headers
		);

		logger.debug({ success, status, body }, "[ACTION] update action result");

		return { success, status, body };
	},

	merge: async ({ cookies, request, fetch, getClientAddress }) => {
		const authHeader = await verifyTokenOnAction(
			cookies,
			request,
			getClientAddress,
			fetch
		);
		if (!authHeader) {
			redirect(302, "/admin/login");
		}

		const headers = buildAuthHeaders();
		const { success, status, body } = await runMerge(fetch, headers);

		logger.debug({ success, status, body }, "[ACTION] merge action result");

		return { success, status, body };
	},
} satisfies Actions;

const UPDATE_API_ROUTE = `${Rh.proto}://${Rh.api}/update`;

async function runUpdate(
	keytype: "channel" | "chatter",
	data: { current: string; historic: string[] },
	fetch: typeof globalThis.fetch,
	headers: Headers
) {
	const updateEndpoint = `${UPDATE_API_ROUTE}/${keytype}`;
	console.log(updateEndpoint);

	const res = await fetch(updateEndpoint, {
		method: "POST",
		headers,
        keepalive: true,
		body: JSON.stringify(data),
	});

	if (!res.ok) {
		console.log(res);

		logger.error({ response: res }, "[ACTION] update failed");
		return { success: false, body: "", status: res.status };
	}

	console.log("response:", res);

	const body = await res.json();
	logger.info({ body }, "RX from server");

	return {
		success: body === "OK",
		status: res.status,
		body,
	};
}

async function runMerge(fetch: typeof globalThis.fetch, headers: Headers) {
	const mergeEndpoint = `${UPDATE_API_ROUTE}/migrate`;
	const res = await fetch(mergeEndpoint, {
		method: "GET",
        keepalive: true,
		headers,
	});

	if (!res.ok) {
		logger.error({ response: res }, "[ACTION] update failed");
		return { success: false, status: res.status, body: "" };
	}

	const body = await res.json();
	logger.info({ body }, "RX from server");

	return {
		success: true,
		status: res.status,
		body,
	};
}
