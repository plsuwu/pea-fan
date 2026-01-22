import type { Instrumentation } from "@opentelemetry/instrumentation";
import { NodeSDK } from "@opentelemetry/sdk-node";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";
import { OTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http";
import { OTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-http";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-node";
import { BatchLogRecordProcessor } from "@opentelemetry/sdk-logs";
import { getNodeAutoInstrumentations } from "@opentelemetry/auto-instrumentations-node";
import {
	resourceFromAttributes,
	type Resource
} from "@opentelemetry/resources";
import { OTEL_EXPORTER_OTLP_ENDPOINT } from "$env/static/private";
import { ATTR_DEPLOYMENT_ENVIRONMENT_NAME } from "@opentelemetry/semantic-conventions/incubating";
import {
	ATTR_SERVICE_NAME,
	ATTR_SERVICE_VERSION
} from "@opentelemetry/semantic-conventions";
import {
	PUBLIC_CLIENT_SERVICE_NAME,
	PUBLIC_CLIENT_SERVICE_VERSION
} from "$env/static/public";
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics";

export class ObservabilityManager {
	#isRunning: boolean = false;
	#sdkInstance: NodeSDK | null = null;
	#traceExporter: OTLPTraceExporter;
	#logExporter: OTLPLogExporter;
	#metricExporter: OTLPMetricExporter;
	#resource: Resource;
	#instrumentations: Array<Instrumentation | Instrumentation[]>;

	constructor() {
		this.#traceExporter = new OTLPTraceExporter({
			url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/traces`,
            headers: {},
		});

		this.#logExporter = new OTLPLogExporter({
			url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/logs`,
            headers: {},
		});

		this.#metricExporter = new OTLPMetricExporter({
			url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/metrics`,
            headers: {},
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

	private makeMetricsReaders(
		exportInterval = 15000
	): Array<PeriodicExportingMetricReader> {
		const reader = new Array(
			new PeriodicExportingMetricReader({
				exporter: this.#metricExporter,
				exportIntervalMillis: exportInterval,
				exportTimeoutMillis: 10000
			})
		);

		return reader;
	}

	start(): void {
		if (this.#isRunning || this.#sdkInstance) {
			console.warn("OTEL SDK already running");
			return;
		}

		this.#sdkInstance = new NodeSDK({
			resource: this.#resource,
			instrumentations: this.#instrumentations,
			spanProcessors: this.makeSpanProcessors(),
			logRecordProcessors: this.makeLogRecordProcessors(),
			metricReaders: this.makeMetricsReaders()
		});


		this.#sdkInstance.start();
		this.#isRunning = true;


		console.debug("OTEL NodeSDK started");
	}

	shutdown(): Promise<void> | undefined {
		if (!this.#sdkInstance) {
			return undefined;
		}

		return this.#sdkInstance?.shutdown();
	}
}
