#!/usr/bin/env bash

set -eux pipefail

mkdir -p ./obsv-data/{tempo,loki,prometheus,grafana}

sudo chown -R 10001:10001 obsv-data/loki
sudo chmod -R 755 obsv-data/loki

sudo chown -R 65534:65534 obsv-data/prometheus
sudo chmod -R 755 obsv-data/prometheus

sudo chown -R 10001:10001 obsv-data/tempo
sudo chmod -R 755 obsv-data/tempo

sudo chown -R 472:472 obsv-data/grafana
sudo chmod -R 755 obsv-data/grafana

docker network create otel-ingest

docker compose up -d --remove-orphans && reset
