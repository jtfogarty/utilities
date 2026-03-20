
I have SurrealDB v3 running as a service on port 8000;

```
sudo ss -tulnp | grep :8000
tcp   LISTEN 0      4096               0.0.0.0:8000       0.0.0.0:*    users:(("surreal",pid=8528,fd=16))
```
now I want to run SurrealMCP in a container and connect to it. via another server on the network.
This MCP server should connect to the SurrealDB v3 service on port 8000.

Is the below command correct?

```bash
# Note: Ensure to replace SURREALDB_USER, SURREALDB_PASS, SURREALDB_NS, and SURREALDB_DB with your actual credentials if different.
# -p 9000:8000 maps the container port 8000 to host port 9000 (change 9000 if needed)
podman run -d --rm --pull=always \
  --name surrealmcp \
  -e SURREALDB_URL=ws://10.10.3.8:8000/rpc \
  -e SURREALDB_USER=root \
  -e SURREALDB_PASS=18b85e792e70178281a2efc29d92e733 \
  -e SURREALDB_NS=salt_agent \
  -e SURREALDB_DB=master \
  -p 9000:8000 \
  docker.io/surrealdb/surrealmcp:latest start --bind-address 0.0.0.0:8000 --server-url http://10.10.3.8:9000 --auth-disabled
```

## How to Test the SurrealMCP Server

1. **Test the HTTP Health Endpoint:**
   To quickly verify if the container is up and responding on port 9000, run this `curl` command:
   ```bash
   curl -v http://10.10.3.8:9000/health
   ```
   You should see `< HTTP/1.1 200 OK` in the output! (I actually just tested this for you, and it is currently returning 200 OK ✅).

2. **Test MCP Capabilities using the MCP Inspector:**
   **Important Note on CORS**: The SurrealMCP container (as of its current version) does not inherently respond to preflight `OPTIONS` requests, which means if you try to run the MCP Inspector on your Mac connecting to the Linux server IP (`10.10.3.8`), the web browser will block the connection due to cross-origin policies.

   To test it interactively using the visual Inspector, you have two simple workarounds:

   **Option A: Run Inspector inside a Container on the Linux Host**
   You can run the MCP inspector directly on the `10.10.3.8` server in another container on the host network, so it accesses SurrealMCP as `localhost`:
   ```bash
   # Run this on the 10.10.3.8 server
   podman run --rm -it --network=host docker.io/node:20-alpine npx -y @modelcontextprotocol/inspector --transport sse --server-url http://localhost:9000/mcp/sse
   ```
   *Then, open the URL it provides in your browser!*

   **Option B: SSH Port Forwarding (Recommended)**
   If you want to run `npx` locally on your Mac, you can temporarily bridge your Mac's localhost to the server using an SSH tunnel:
   ```bash
   # Run this on your Mac terminal to map your local port 9000 to the server's port 9000
   ssh -L 9000:localhost:9000 your_user@10.10.3.8 -N
   ```
   *Keep that terminal open, then in a new terminal on your Mac, run:*
   ```bash
   npx @modelcontextprotocol/inspector --transport sse --server-url http://localhost:9000/mcp/sse
   ```