import { PUBLIC_CLIENT_SERVICE_NAME } from "$env/static/public";
import pino from "pino";

const transports = pino.transport({
	targets: [
		{
			target: "pino-opentelemetry-transport",
			level: "trace",
			options: {
				serviceName: PUBLIC_CLIENT_SERVICE_NAME,
				logLevel: "trace"
			}
		},
		{
			target: "pino-pretty",
			level: "trace",
			options: {
				colorize: true,
				translateTime: "HH:MM:ss",
				ignore: "pid"
			}
		}
	]
});
export const logger = pino(transports);

