#!/usr/bin/env bash


curl -X POST http://localhost:4318/v1/logs  \
    -H "Content-Type: application/json"     \
    -d '{ 
            "resourceLogs": [{
            "resource": { "attributes": [{"key": "service.name", "value": {"stringValue": "test-service"}}] },
            "scopeLogs": [{
                "logRecords": [{
                    "timeUnixNano": "'$(date +%s)000000000'",
                    "body": {"stringValue": "Test log message from OTLP"},
                    "severityText": "INFO" 
                }] 
            }]
        }]
    }'


