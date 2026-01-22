import pino from "pino";
import { OTEL_LOKI_HTTP } from "$env/static/private";
import { PUBLIC_CLIENT_SERVICE_NAME } from "$env/static/public";

// const targets = new Array();
// targets.push({
// 	target: "pino-pretty",
// 	options: {
// 		colorize: true,
// 		translateTime: "HH:MM:ss",
// 		ignore: "pid"
// 	}
// });
//
// targets.push({
// 	target: "pino-loki",
// 	level: "trace",
// 	options: {
// 		batching: { interval: 5 },
// 		host: OTEL_LOKI_HTTP,
// 		labels: {
// 			service_name: PUBLIC_CLIENT_SERVICE_NAME,
// 			environment: import.meta.env.DEV ? "development" : "production"
// 		}
// 	}
// });

export const transport = pino.transport({
	targets: [
		{
			target: "pino-pretty",
			level: "trace",
			options: {
				colorize: true,
				translateTime: "HH:MM:ss",
				ignore: "pid"
			}
		},
		{
			target: "pino-loki",
			level: "trace",
			options: {
				batching: { interval: 5 },
				host: OTEL_LOKI_HTTP,
				labels: {
					service_name: PUBLIC_CLIENT_SERVICE_NAME,
					environment: import.meta.env.DEV ? "development" : "production"
				}
			}
		}
	]
});
export const logger = pino(transport);
export type Logger = typeof logger;

logger.debug("pino transport setup ok");
