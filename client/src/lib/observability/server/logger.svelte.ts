import {
	PUBLIC_CLIENT_SERVICE_NAME,
	PUBLIC_CLIENT_SERVICE_VERSION
} from "$env/static/public";
import pino, { type Logger, type LoggerOptions } from "pino";
import type { Cache } from "./cache.svelte";
import type { Span } from "@opentelemetry/api";
import type { LogEntry, TelemetryPayload } from "../types";
import type { ReadableSpan } from "@opentelemetry/sdk-trace-base";
import { traced } from "./tracing";
import type {
	ChannelEntry,
	ChatterEntry,
	Entry,
	PaginatedResponse,
	Score
} from "$lib/types";

export type SerializableClientLogBatch = {
	traceId: string;
	spanId: string;
	entry: LogEntry;
	url: string;
	userAgent: string;
};

function isReadableSpan(span: Span): span is Span & ReadableSpan {
	return "name" in span && "attributes" in span && "status" in span;
}

export const serializeHandlers = {
	response: (res: Response) => {
		return {
			status: res.status,
			statusText: res.statusText,
			headers: Object.fromEntries(res.headers)
		};
	},

	cache: (cache: Cache<any>, all = false) => {
		if (all) {
			return {
				cachesFor: cache.url,
				currentTime: new Date().toLocaleString(),
				lastRefresh: cache.lastRefresh,
				nextRefresh: cache.nextRefresh,
				data: cache.data,
				ttl: cache.ttl
			};
		} else {
			const slice = cache.data.slice(0, 8);
			return {
				currentTime: new Date().toLocaleString(),
				lastRefresh: cache.lastRefresh,
				nextRefresh: cache.nextRefresh,
				data:
					Array.isArray(cache.data) && cache.data.length > 0
						? [...slice, `...(${cache.data.length - slice.length} more)`]
						: []
			};
		}
	},

	span: (span: Span) => {
		const spanCtx = span.spanContext();
		const serialized: Record<string, unknown> = {
			traceId: spanCtx.traceId,
			spanId: spanCtx.spanId,
			traceFlags: spanCtx.traceFlags,
			isRemote: spanCtx.isRemote
		};

		if (isReadableSpan(span)) {
			serialized.name = span.name;
			serialized.kind = span.kind;
			serialized.status = {
				code: span.status.code,
				message: span.status.message
			};
			serialized.attributes = span.attributes;
			serialized.duration = span.duration;
			serialized.startTime = span.startTime;
			serialized.endTime = span.endTime;

			if (span.events.length > 0) {
				serialized.events = span.events.map((e) => ({
					name: e.name,
					time: e.time,
					attributes: e.attributes
				}));
			}
		}

		return serialized;
	},

	entry: (entry: Entry) => {
		if (entry._tag === "Channel") {
			const len = entry.data.chatter_scores?.length || 0;
			const scores =
				entry.data.chatter_scores?.slice(0, 5).map((s) => {
					const score: Score = { _tag: "Chatter", data: s };
					return serializeHandlers.score(score);
				}) || [];
			return {
				channel: `${entry.data.login}:${entry.data.id}`,
				scores: scores.length == 0 ? [] : [...scores, `...(${len - 5} more)`],
				totalChannel: entry.data.total_channel
			};
		} else if (entry._tag === "Chatter") {
			const len = entry.data.channel_scores?.length || 0;
			const scores =
				entry.data.channel_scores?.slice(0, 5).map((s) => {
					const score: Score = { _tag: "Channel", data: s };
					return serializeHandlers.score(score);
				}) || [];

			return {
				chatter: `${entry.data.login}:${entry.data.id}`,
				scores: scores.length == 0 ? [] : [...scores, `...(${len - 5} more)`],
				total: entry.data.total
			};
		}
	},

	score: (score: Score) => {
		if (score._tag === "Channel") {
			return `${score.data.channel_login} -> ${score.data.score}`;
		} else if (score._tag === "Chatter") {
			return `${score.data.chatter_login} -> ${score.data.score}`;
		}
	},

	leaderboard: ({
		leaderboard,
		variant
	}: {
		leaderboard: PaginatedResponse;
		variant: "chatter" | "channel";
	}) => {
		const leaderboardSlice = new Array();
		if (!leaderboard || !leaderboard.items) {
			return "empty_leaderboard";
		}

		if (variant === "chatter") {
			leaderboard.items?.slice(0, 3).forEach((item) => {
				const entry: Entry = {
					_tag: "Chatter",
					data: item as ChatterEntry
				};

				leaderboardSlice.push(serializeHandlers.entry(entry));
			});
		} else {
			leaderboard.items?.slice(0, 3).forEach((item) => {
				const entry: Entry = {
					_tag: "Channel",
					data: item as ChannelEntry
				};

				leaderboardSlice.push(serializeHandlers.entry(entry));
			});
		}

		return {
			pageSize: leaderboard.page_size,
			currentPage: leaderboard.page,
			totalPages: leaderboard.total_pages,
			items: leaderboardSlice,
			totalItems: leaderboard.total_items
		};
	},

	clientLogEntry: ({
		span,
		entry,
		payload,
		addr
	}: {
		span: Span;
		entry: LogEntry;
		payload: Partial<TelemetryPayload>;
		addr: string;
	}) => {
		const serializedSpan = serializeHandlers.span(span);
		return {
			...serializedSpan,
			url: payload.url,
			userAgent: payload.userAgent,
			clientId: payload.clientId,
			sessionId: payload.sessionId,
			clientAddr: addr,
			timestamp: entry.timestamp,
			source: "client"
		};
	}
};

const pinoLogger = (() => {
	const targets = new Array();
	targets.push({
		target: "pino-opentelemetry-transport",
		options: {
			serviceVersion: PUBLIC_CLIENT_SERVICE_VERSION
		},
		level: "trace"
	});

	targets.push({
		target: "pino-pretty",
		options: {
			colorize: true,
			translateTime: true,
			ignore: "pid,hostname"
		},
		level: import.meta.env.DEV ? "trace" : "info"
	});

	let pinoOptions: LoggerOptions = {
		transport: { targets },
		serializers: {
			// full details about an in-memory cache
			cacheAll: (cache) => serializeHandlers.cache(cache, true),

			// summary of an in-memory cache
			cache: (cache) => serializeHandlers.cache(cache),
			span: (span) => serializeHandlers.span(span),
			response: (res: Response) => serializeHandlers.response(res),
			leaderboard: ({ leaderboard, variant }) =>
				serializeHandlers.leaderboard({ leaderboard, variant }),
			clientLogEntry: ({ span, entry, payload, addr }) => {
				return serializeHandlers.clientLogEntry({
					span,
					entry,
					payload,
					addr
				});
			},

			error: pino.stdSerializers.err
		}
	};

	return pino(pinoOptions);
})();

class PinoLogger {
	public logger: Logger | undefined = $state(undefined);

	constructor(logger: Logger) {
		this.logger = logger;
	}
}

const loggerStore = new PinoLogger(pinoLogger);
loggerStore.logger!.level = "trace";

export const logger = loggerStore.logger!;
