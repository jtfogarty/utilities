#!/bin/bash
# Install and run SurrealDB v3 via Podman for Agent Memory

# Start dedicated SurrealDB v3.x for the agent
podman rm -f surrealagentdb 2>/dev/null || true
podman run -d --name surrealagentdb -p 8089:8000 \
  docker.io/surrealdb/surrealdb:latest start --bind 0.0.0.0:8000 --user root --pass 18b85e792e70178281a2efc29d92e733

# Wait for DB to start up
sleep 2

echo "SurrealDB v3 Agent Engine started on port 8089."
