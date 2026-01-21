import type { Instrumentation } from "@opentelemetry/instrumentation";
import { NodeSDK } from "@opentelemetry/sdk-node";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-proto";
import { OTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-http";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-node";
import { BatchLogRecordProcessor } from "@opentelemetry/sdk-logs";
import { diag, DiagConsoleLogger, DiagLogLevel } from "@opentelemetry/api";
import { getNodeAutoInstrumentations } from "@opentelemetry/auto-instrumentations-node";
import { createAddHookMessageChannel } from "import-in-the-middle";
import { register } from "node:module";
import { logger } from "./server/logger";
import {
	resourceFromAttributes,
	type Resource
} from "@opentelemetry/resources";
import { OTEL_TEMPO_HTTP, OTEL_LOKI_HTTP } from "$env/static/private";
import { ATTR_DEPLOYMENT_ENVIRONMENT_NAME } from "@opentelemetry/semantic-conventions/incubating";
import {
	ATTR_SERVICE_NAME,
	ATTR_SERVICE_VERSION
} from "@opentelemetry/semantic-conventions";
import {
	PUBLIC_CLIENT_SERVICE_NAME,
	PUBLIC_CLIENT_SERVICE_VERSION
} from "$env/static/public";

diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.ALL);

export class ObservabilityManager {
	#isRunning: boolean = false;
	#sdkInstance: NodeSDK | null = null;
	#traceExporter: OTLPTraceExporter;
	#logExporter: OTLPLogExporter;
	#resource: Resource;
	#instrumentations: Array<Instrumentation | Instrumentation[]>;

	constructor() {
		this.#traceExporter = new OTLPTraceExporter({
			url: OTEL_TEMPO_HTTP
		});

		this.#logExporter = new OTLPLogExporter({
			url: OTEL_LOKI_HTTP
		});

		this.#resource = resourceFromAttributes({
			[ATTR_SERVICE_NAME]: PUBLIC_CLIENT_SERVICE_NAME || "sveltekit-client",
			[ATTR_SERVICE_VERSION]: PUBLIC_CLIENT_SERVICE_VERSION || "0.0.1",
			[ATTR_DEPLOYMENT_ENVIRONMENT_NAME]: import.meta.env.DEV
				? "development"
				: "production"
		});

		this.#instrumentations = this.createInstrumentations();
	}

	withTraceExporter(url: string): this {
		this.#traceExporter = new OTLPTraceExporter({ url });
		return this;
	}
	withLogExporter(url: string): this {
		this.#logExporter = new OTLPLogExporter({ url });
		return this;
	}

	withResource(
		serviceName: string,
		serviceVersion: string,
		environment: string
	): this {
		this.#resource = resourceFromAttributes({
			[ATTR_SERVICE_NAME]: serviceName || PUBLIC_CLIENT_SERVICE_NAME,
			[ATTR_SERVICE_VERSION]: serviceVersion || PUBLIC_CLIENT_SERVICE_VERSION,
			[ATTR_DEPLOYMENT_ENVIRONMENT_NAME]:
				environment ?? (import.meta.env.DEV ? "development" : "production")
		});
		return this;
	}

	withInstrumentations(ignoreEndpoints: Array<string> = ["/favicon"]): this {
		this.#instrumentations = this.createInstrumentations(ignoreEndpoints);
		return this;
	}

	makeLogRecordProcessors(
		maxQueueSize = 1024,
		scheduledDelayMillis = 5000
	): Array<BatchLogRecordProcessor> {
		return new Array(
			new BatchLogRecordProcessor(this.#logExporter, {
				maxQueueSize,
				scheduledDelayMillis
			})
		);
	}

	makeSpanProcessors(
		maxQueueSize = 1024,
		scheduledDelayMillis = 5000
	): Array<BatchSpanProcessor> {
		return new Array(
			new BatchSpanProcessor(this.#traceExporter, {
				maxQueueSize,
				scheduledDelayMillis
			})
		);
	}

	private createInstrumentations(
		ignoreEndpoints: Array<string> = ["/favicon"]
	): Array<Instrumentation | Instrumentation[]> {
		const instrumentations = new Array(
			getNodeAutoInstrumentations({
				"@opentelemetry/instrumentation-fs": { enabled: false },
				"@opentelemetry/instrumentation-http": {
					ignoreIncomingRequestHook: (req) => {
						const url = req.url || "";
						return ignoreEndpoints.some((endpoint) => url.includes(endpoint));
					}
				}
			})
		);

		return instrumentations;
	}

	start(): void {
		if (this.#isRunning) {
			logger.warn({ resource: this.#resource }, "OTEL NodeSDK already running");
			return;
		}

		const { registerOptions } = createAddHookMessageChannel();
		register("import-in-the-middle/hook.mjs", import.meta.url, registerOptions);

		this.#sdkInstance = new NodeSDK({
			resource: this.#resource,
			spanProcessors: this.makeSpanProcessors(),
			logRecordProcessors: this.makeLogRecordProcessors(),
			instrumentations: this.#instrumentations
		});

		this.#sdkInstance.start();
		this.#isRunning = true;

		logger.debug("OTEL NodeSDK started");
	}

	shutdown(): Promise<void> | undefined {
		if (!this.#sdkInstance) {
			return undefined;
		}

		return this.#sdkInstance?.shutdown();
	}
}
