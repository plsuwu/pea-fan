#!/usr/bin/env bash

export LOGGING_ENABLE_ALL=1

LOCAL_VOLUME=${PWD}/.container
RELEASE=${1:-latest}
RUNTIME=docker
MOUNT_OPTS=rw

for dir in grafana prometheus loki; do
	test -d "${LOCAL_VOLUME}"/${dir} || mkdir -p "${LOCAL_VOLUME}"/${dir}
done

IMAGE="docker.io/grafana/otel-lgtm:${RELEASE}"
$RUNTIME image pull "$IMAGE"


$RUNTIME container run \
	--name lgtm \
	-p 3100:3000 \
	-p 4040:4040 \
	-p 4317:4317 \
	-p 4318:4318 \
	-p 9090:9090 \
	--rm \
	-ti \
	-v "${LOCAL_VOLUME}"/grafana:/data/grafana:"${MOUNT_OPTS}" \
	-v "${LOCAL_VOLUME}"/prometheus:/data/prometheus:"${MOUNT_OPTS}" \
	-v "${LOCAL_VOLUME}"/loki:/data/loki:"${MOUNT_OPTS}" \
	-e GF_PATHS_DATA=/data/grafana \
	--env-file .env.lgtm \
	"$IMAGE"
