import pino from "pino";
import { OTEL_LOKI_HTTP } from "$env/static/private";
import { LOG_LEVEL } from "../types";
import { PUBLIC_CLIENT_SERVICE_NAME } from "$env/static/public";

const targets = new Array();
if (import.meta.env.DEV) {
	targets.push({
		target: "pino-pretty",
		options: {
			colorize: true,
			translateTime: "HH:MM:ss",
			ignore: "pid"
		}
	});
}

targets.push({
	target: "pino-loki",
	options: {
		levelMap: LOG_LEVEL,
		batching: true,
		interval: 5,
		host: OTEL_LOKI_HTTP || "http://localhost:3100",
		labels: {
			service_name: PUBLIC_CLIENT_SERVICE_NAME,
			environment: import.meta.env.DEV ? "development" : "production"
		}
	}
});

export const transport = pino.transport({ targets });
export const logger = pino(transport);
export type Logger = typeof logger;

logger.debug("pino transport setup ok");
