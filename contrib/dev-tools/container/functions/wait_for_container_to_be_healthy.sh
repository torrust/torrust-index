#!/bin/bash

wait_for_container_to_be_healthy() {
    local container_name="$1"
    local max_retries="$2"
    local retry_interval="$3"
    local retry_count=0

    while [ $retry_count -lt "$max_retries" ]; do
        container_health="$(docker inspect --format='{{json .State.Health}}' "$container_name")"
        if [ "$container_health" != "{}" ]; then
            container_status="$(echo "$container_health" | jq -r '.Status')"
            if [ "$container_status" == "healthy" ]; then
                echo "Container $container_name is healthy"
                return 0
            fi
        fi

        retry_count=$((retry_count + 1))
        echo "Waiting for container $container_name to become healthy (attempt $retry_count of $max_retries)..."
        sleep "$retry_interval"
    done

    echo "Timeout reached, container $container_name is not healthy"
    return 1
}