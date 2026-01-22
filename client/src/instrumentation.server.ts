import { createAddHookMessageChannel } from "import-in-the-middle";
import { register } from "node:module";
const { registerOptions, waitForAllMessagesAcknowledged } =
	createAddHookMessageChannel();
register("import-in-the-middle", import.meta.url, registerOptions);
await waitForAllMessagesAcknowledged();

import { diag, DiagConsoleLogger, DiagLogLevel } from "@opentelemetry/api";
diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.INFO);

import { NodeSDK } from "@opentelemetry/sdk-node";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-node";
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics";
import { BatchLogRecordProcessor } from "@opentelemetry/sdk-logs";
import { resourceFromAttributes } from "@opentelemetry/resources";
import { OTEL_EXPORTER_OTLP_ENDPOINT } from "$env/static/private";
import { ATTR_DEPLOYMENT_ENVIRONMENT_NAME } from "@opentelemetry/semantic-conventions/incubating";

import { HttpInstrumentation } from "@opentelemetry/instrumentation-http";
import { PinoInstrumentation } from "@opentelemetry/instrumentation-pino";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";
import { OTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http";
import { OTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-http";
import {
	PUBLIC_CLIENT_SERVICE_NAME,
	PUBLIC_CLIENT_SERVICE_VERSION
} from "$env/static/public";
import {
	ATTR_SERVICE_NAME,
	ATTR_SERVICE_VERSION
} from "@opentelemetry/semantic-conventions";

const DEFAULT_RESOURCE = resourceFromAttributes({
	[ATTR_SERVICE_NAME]: PUBLIC_CLIENT_SERVICE_NAME ?? "client",
	[ATTR_SERVICE_VERSION]: PUBLIC_CLIENT_SERVICE_VERSION ?? "0.0.1",
	[ATTR_DEPLOYMENT_ENVIRONMENT_NAME]: import.meta.env.DEV
		? "development"
		: "production"
});

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
	resource: DEFAULT_RESOURCE,
	spanProcessors: [new BatchSpanProcessor(SPAN_EXPORTER)],
	logRecordProcessors: [new BatchLogRecordProcessor(LOG_EXPORTER)],
	metricReaders: [
		new PeriodicExportingMetricReader({
			exporter: METRIC_EXPORTER
		})
	],
	instrumentations: [
		new PinoInstrumentation({
			logHook: (_, record) => {
				record["service_name"] = DEFAULT_RESOURCE.attributes["service.name"];
			},
			disableLogCorrelation: false,
			disableLogSending: false
		}),
		new HttpInstrumentation()
	]
});

sdk.start();
