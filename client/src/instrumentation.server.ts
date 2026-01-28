import { createAddHookMessageChannel } from "import-in-the-middle";
import { register } from "node:module";
const { registerOptions, waitForAllMessagesAcknowledged } =
	createAddHookMessageChannel();
register("import-in-the-middle", import.meta.url, registerOptions);
await waitForAllMessagesAcknowledged();

// import { diag, DiagConsoleLogger, DiagLogLevel } from "@opentelemetry/api";
// diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.INFO);

import { NodeSDK } from "@opentelemetry/sdk-node";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-node";
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics";
import { BatchLogRecordProcessor } from "@opentelemetry/sdk-logs";
import { resourceFromAttributes } from "@opentelemetry/resources";
import { OTEL_EXPORTER_OTLP_ENDPOINT } from "$env/static/private";

import { HttpInstrumentation } from "@opentelemetry/instrumentation-http";
// import { PinoInstrumentation } from "@opentelemetry/instrumentation-pino";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";
import { OTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http";
import { OTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-http";
import {
	PUBLIC_CLIENT_SERVICE_NAME,
	PUBLIC_CLIENT_SERVICE_VERSION
} from "$env/static/public";

const SPAN_EXPORTER = new OTLPTraceExporter({
	url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/traces`,
	headers: {}
});

const METRIC_EXPORTER = new OTLPMetricExporter({
	url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/metrics`,
	headers: {}
});

const LOG_EXPORTER = new OTLPLogExporter({
	url: `${OTEL_EXPORTER_OTLP_ENDPOINT}/v1/logs`,
	headers: {}
});

let sdk = new NodeSDK({
	serviceName: PUBLIC_CLIENT_SERVICE_NAME,
	resource: resourceFromAttributes({
		"service.name": PUBLIC_CLIENT_SERVICE_NAME || "piss-fan-client",
		"service.version": PUBLIC_CLIENT_SERVICE_VERSION || "0.0.1",
		"deployment.environment.name": import.meta.env.DEV
			? "development"
			: "production"
	}),
	spanProcessors: [new BatchSpanProcessor(SPAN_EXPORTER)],
	logRecordProcessors: [new BatchLogRecordProcessor(LOG_EXPORTER)],
	metricReaders: [
		new PeriodicExportingMetricReader({
			exporter: METRIC_EXPORTER
		})
	],
	instrumentations: [
		// new PinoInstrumentation({
		// 	disableLogCorrelation: false,
		// 	disableLogSending: false,
		// 	logHook: (span, record) => {
		// 		record["traceId"] = span.spanContext().traceId;
		// 		record["spanId"] = span.spanContext().spanId;
		// 	}
		// }),
		new HttpInstrumentation()
	]
});

const shutdown = () => {
	process.removeAllListeners();
	let status = 0;

	sdk
		.shutdown()
		.then(() => console.log("tracing terminated"))
		.catch((error) => {
			console.log("error during tracing termination", error);
			status = 1;
		})
		.finally(() => process.exit(status));
};

sdk.start();

process.on("SIGTERM", shutdown);
process.on("SIGINT", shutdown);
