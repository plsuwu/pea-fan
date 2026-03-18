import { createAddHookMessageChannel } from "import-in-the-middle";
import { register } from "node:module";

// register module loader hook with `import-in-the-middle` to intercept ESM imports,
// making their exports mutable such that instrumentations can patch them
const { registerOptions, waitForAllMessagesAcknowledged } =
	createAddHookMessageChannel();
register("import-in-the-middle", import.meta.url, registerOptions);

await waitForAllMessagesAcknowledged(); // ensure loader hook is fully registered

import { NodeSDK } from "@opentelemetry/sdk-node";
import { getNodeAutoInstrumentations } from "@opentelemetry/auto-instrumentations-node";
import { BatchSpanProcessor } from "@opentelemetry/sdk-trace-node";
import { PeriodicExportingMetricReader } from "@opentelemetry/sdk-metrics";
import { BatchLogRecordProcessor } from "@opentelemetry/sdk-logs";
import { OTLPTraceExporter } from "@opentelemetry/exporter-trace-otlp-http";
import { OTLPMetricExporter } from "@opentelemetry/exporter-metrics-otlp-http";
import { OTLPLogExporter } from "@opentelemetry/exporter-logs-otlp-http";

import packageJson from "../package.json";
import { resourceFromAttributes } from "@opentelemetry/resources";
import {
	ATTR_SERVICE_NAME,
	ATTR_SERVICE_VERSION,
} from "@opentelemetry/semantic-conventions";

const ATTR_DEPLOYMENT_ENVIRONMENT = "deployment.environment.name";
const DEPLOYMENT_ENVIRONMENT = import.meta.env.MODE;
const SERVICE_NAME = packageJson.name;
const SERVICE_VERSION = packageJson.version;

const INSTRUMENTATION_IGNORE_ENDPOINTS = ["/favico"];

const otlpUrl = (path: string) =>
	`http://localhost:4318/v1/${path}`;

const sdk = new NodeSDK({
	resource: resourceFromAttributes({
		[ATTR_SERVICE_NAME]: SERVICE_NAME,
		[ATTR_SERVICE_VERSION]: SERVICE_VERSION,
		[ATTR_DEPLOYMENT_ENVIRONMENT]: DEPLOYMENT_ENVIRONMENT,
	}),

	spanProcessors: [
		new BatchSpanProcessor(new OTLPTraceExporter({ url: otlpUrl("traces") })),
	],
	logRecordProcessors: [
		new BatchLogRecordProcessor(new OTLPLogExporter({ url: otlpUrl("logs") })),
	],
	metricReaders: [
		new PeriodicExportingMetricReader({
			exporter: new OTLPMetricExporter({ url: otlpUrl("metrics") }),
		}),
	],
	instrumentations: [
		getNodeAutoInstrumentations({
			"@opentelemetry/instrumentation-http": {
				ignoreIncomingRequestHook: (req) => {
					const url = req.url || "";
					return INSTRUMENTATION_IGNORE_ENDPOINTS.some((endpoint) =>
						url.includes(endpoint)
					);
				},
			},
		}),
	],
});

sdk.start();

const shutdown = () => {
	process.off("SIGTERM", shutdown);
	process.off("SIGINT", shutdown);

	sdk
		.shutdown()
		.then(() => console.log("tracing terminated"))
		.catch((err) => {
			console.error("error during tracing termination", err);
			process.exitCode = 1;
		})
		.finally(() => process.exit());
};

process.on("SIGTERM", shutdown);
process.on("SIGINT", shutdown);
