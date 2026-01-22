import { browser } from "$app/environment";
import type {
	LogEntry,
	LogLevel,
	LogContext,
	TelemetryPayload
} from "../types";
import { LOG_LEVEL } from "../types";

interface ClientLoggerConfig {
	level: LogLevel;
	endpoint: string;
	batchSize: number;
	flushInterval: number;
	maxRetries: number;
}

const DEFAULT_CONFIG: ClientLoggerConfig = {
	level: "debug",
	endpoint: "/api/otel",
	batchSize: 1024,
	flushInterval: 10000,
	maxRetries: 3
};

const CLIENT_STORED = "_otel_client";

export class ClientLogger {
	private buffer: Array<LogEntry> = new Array();
	private config: ClientLoggerConfig;
	private flushTimer: ReturnType<typeof setInterval> | null = null;
	private context: LogContext = {};
	private clientId: string;
	private sessionId: string;

	constructor(config: Partial<ClientLoggerConfig> = {}) {
		this.config = { ...DEFAULT_CONFIG, ...config };
		this.clientId = this.getOrCreateClientId();
		this.sessionId = this.createSessionId();

		if (browser) {
			this.startFlushTimer();
			this.setupUnloadHandler();
		}
	}

	private getOrCreateClientId(): string {
		if (!browser) return "server";

		let clientId = localStorage.getItem(CLIENT_STORED);
		if (!clientId) {
			clientId = crypto.randomUUID();
			localStorage.setItem(CLIENT_STORED, clientId);
		}

		return clientId;
	}

	private createSessionId(): string {
		return crypto.randomUUID();
	}

	private startFlushTimer(): void {
		this.flushTimer = setInterval(
			() => this.flush(),
			this.config.flushInterval
		);
	}

	private setupUnloadHandler(): void {
		window.addEventListener("beforeunload", () => {
			if (document.visibilityState === "hidden") {
				this.flush(true);
			}
		});
	}

	setContext(ctx: LogContext): void {
		this.context = { ...this.context, ...ctx };
	}

	clearContext(): void {
		this.context = {};
	}

	private shouldLog(level: LogLevel): boolean {
		return LOG_LEVEL[level] >= LOG_LEVEL[this.config.level];
	}

	private log(
		level: LogLevel,
		message: string,
		data?: Record<string, unknown>
	): void {
		if (!this.shouldLog(level)) {
			return;
		}

		const entry: LogEntry = {
			level,
			message,
			timestamp: Date.now(),
			context: { ...this.context },
			...data
		};

		this.buffer.push(entry);

		if (import.meta.env.DEV) {
			const consoleFn = level === "fatal" ? "error" : level;
			console[consoleFn as "log"](`[${level.toUpperCase()}]`, message + "\n", {
				data
			});
		}

		if (this.buffer.length >= this.config.batchSize) {
			this.flush();
		}
	}

	async flush(sync = false): Promise<void> {
		if (this.buffer.length === 0) {
			return;
		}

		const logs = [...this.buffer];
		this.buffer = new Array();

		const payload: TelemetryPayload = {
			logs,
			clientId: this.clientId,
			sessionId: this.sessionId,
			url: browser ? window.location.href : "",
			userAgent: browser ? navigator.userAgent : ""
		};

		if (sync && browser && "sendBeacon" in navigator) {
			navigator.sendBeacon(this.config.endpoint, JSON.stringify(payload));
			return;
		}

		try {
			await this.sendWithRetry(payload);
		} catch (err) {
			console.error(
				"failed to send otel to server; clearing logs.",
				"\nerr:",
				err
			);
		}
	}

	private async sendWithRetry(
		payload: TelemetryPayload,
		attempt = 1
	): Promise<void> {
		try {
			const response = await fetch(this.config.endpoint, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(payload)
			});

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}`);
			} else {
				console.info(`OTEL collection OK`);
			}
		} catch (err) {
			if (attempt <= this.config.maxRetries) {
				const timeout = attempt * 5 * 1000;
				console.log(
					`OTEL send failure; retrying in ${timeout / 1000} secs`,
					`(attempt ${attempt}/${this.config.maxRetries})`
				);
				return await new Promise(() => {
					return setTimeout(
						() => {
							this.sendWithRetry(payload, attempt + 1);
						},
						Math.pow(2, attempt) * 2000
					);
				});
			}

			throw err;
		}
	}

	trace(message: string, data?: Record<string, unknown>): void {
		this.log("trace", message, data);
	}

	debug(message: string, data?: Record<string, unknown>): void {
		this.log("debug", message, data);
	}

	info(message: string, data?: Record<string, unknown>): void {
		this.log("info", message, data);
	}

	warn(message: string, data?: Record<string, unknown>): void {
		this.log("warn", message, data);
	}

	error(message: string, data?: Record<string, unknown>): void {
		this.log("error", message, data);
	}

	fatal(message: string, data?: Record<string, unknown>): void {
		this.log("fatal", message, data);
	}

	child(bindings?: LogContext): ClientLogger {
		const child = new ClientLogger(this.config);
		child.context = { ...this.context, ...bindings };

		// child shares parent's log buffer
		child.buffer = this.buffer;
		return child;
	}

	destroy(): void {
		if (this.flushTimer) {
			clearInterval(this.flushTimer);
		}

		this.flush(true);
	}
}

export const clientLogger = new ClientLogger();
export const trace = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.trace(msg, data);

export const debug = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.debug(msg, data);

export const info = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.info(msg, data);

export const warn = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.warn(msg, data);

export const error = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.error(msg, data);

export const fatal = (msg: string, data?: Record<string, unknown>) =>
	clientLogger.fatal(msg, data);

export default clientLogger;
