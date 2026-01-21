import type { ApiError } from "$lib/types/error";
export const LOG_LEVEL = {
	trace: "trace",
	debug: "debug",
	info: "info",
	warn: "warn",
	error: "error",
	fatal: "fatal"
} as const;

export type LogLevel = (typeof LOG_LEVEL)[keyof typeof LOG_LEVEL];

export interface LogContext {
	traceId?: string;
	spanId?: string;
	userId?: string;
	requestId?: string;
	[key: string]: unknown;
}

export interface LogEntry {
	level: LogLevel;
	message: string;
	timestamp: number;
	context?: LogContext;
	error?: ApiError | Error;
	[key: string]: unknown;
}

export interface TelemetryPayload {
	logs: Array<LogEntry>;
	clientId: string;
	sessionId: string;
	url: string;
	userAgent: string;
}
