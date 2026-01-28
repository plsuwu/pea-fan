export type ApiError = {
	code: string | number;
	message: string;
	details: Record<string, unknown>;
	silent?: boolean;
	displayMessage?: string;
};

export const LOG_LEVEL = {
	fatal: "fatal",
	error: "error",
	warn: "warn",
	info: "info",
	debug: "debug",
	trace: "trace"
} as const;

export type LogLevel = (typeof LOG_LEVEL)[keyof typeof LOG_LEVEL];

export interface LogContext {
	traceId?: string;
	spanId?: string;
	requestId?: string;
	[key: string]: unknown;
}

export interface LogEntry {
	level: LogLevel;
	message: string;
	timestamp: number;
	context?: LogContext;
	error?: Error | string;
	[key: string]: unknown;
}

export interface TelemetryPayload {
	logs: Array<LogEntry>;
	clientId: string;
	sessionId: string;
	url: string;
	userAgent: string;
}
