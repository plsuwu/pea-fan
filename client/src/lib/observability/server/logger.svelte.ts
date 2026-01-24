import { PUBLIC_CLIENT_SERVICE_NAME } from "$env/static/public";
import pino, { type Logger, type LoggerOptions } from "pino";
import type { Cache } from "./cache";

export const serializeHandlers = {
	response: (res: Response) => {
		return {
			status: res.status,
			statusText: res.statusText,
			headers: Object.fromEntries(res.headers)
		};
	},

	cache: (cache: Cache<any>, full = false) => {
		if (full) {
			return {
				caches: cache.url,
				lastRefresh: cache.lastRefresh,
				nextRefresh: cache.nextRefresh,
				channels: cache.channels,
				ttl: cache.ttl
			};
		} else {
			return {
				lastRefresh: cache.lastRefresh,
				nextRefresh: cache.nextRefresh,
				channels:
					cache.channels.length > 0
						? [
								...cache.channels.slice(0, 5),
								`...(${cache.channels.length - 5} more)`
							]
						: []
			};
		}
	}
};

const pinoLogger = (() => {
	if (import.meta.env.DEV) {
		process.loadEnvFile();
	}

	let pinoOptions: LoggerOptions = {
		transport: {
			targets: [
				{
					target: "pino-opentelemetry-transport",
					options: {
						serviceName: PUBLIC_CLIENT_SERVICE_NAME,
						loggerName: PUBLIC_CLIENT_SERVICE_NAME
					},
					level: "trace"
				},
				{
					target: "pino-pretty",
					options: {
						serviceName: PUBLIC_CLIENT_SERVICE_NAME,
						colorize: true,
						translateTime: true,
						ignore: "pid,hostname"
					},
					level: "trace"
				}
			]
		},

		serializers: {
			response: (res: Response) => {
				return serializeHandlers.response(res);
			},

			// full details about an in-memory cache
			cacheFull: (cache) => {
				return serializeHandlers.cache(cache, true);
			},

			// summary of an in-memory cache
			cache: (cache) => {
				return serializeHandlers.cache(cache);
			},

			error: pino.stdSerializers.err
		}
	};

	return pino(pinoOptions);
})();

class PinoLogger {
	public readonly logger: Logger | undefined = $state(undefined);

	constructor(logger: Logger) {
		this.logger = logger;
	}
}

const loggerStore = new PinoLogger(pinoLogger);
loggerStore.logger!.level = "trace";

export const logger = loggerStore.logger!;

// export const logger = pino({
// 	transport: {
// 		targets: [
// 			{
// 				target: "pino-opentelemetry-transport",
// 				options: {
// 					serviceName: PUBLIC_CLIENT_SERVICE_NAME,
// 					loggerName: PUBLIC_CLIENT_SERVICE_NAME
// 				},
// 				level: "trace"
// 			},
// 			{
// 				target: "pino-pretty",
// 				options: {
// 					serviceName: PUBLIC_CLIENT_SERVICE_NAME,
// 					colorize: true,
// 					translateTime: true,
// 					ignore: "pid,hostname"
// 				},
// 				level: "trace"
// 			}
// 		],
// 		level: "trace",
// 		options: {
// 			serviceName: PUBLIC_CLIENT_SERVICE_NAME
// 		}
// 	},
//
// 	serializers: {
// 		response: (res: Response) => {
// 			return serializeHandlers.response(res);
// 		},
//
// 		// full details about an in-memory cache
// 		cacheFull: (cache) => {
// 			return serializeHandlers.cache(cache, true);
// 		},
//
// 		// summary of an in-memory cache
// 		cache: (cache) => {
// 			return serializeHandlers.cache(cache);
// 		},
//
// 		error: pino.stdSerializers.err
// 	}
// });

// export const logger = pino(dest);
