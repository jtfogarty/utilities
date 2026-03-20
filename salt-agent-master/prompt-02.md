# Integrating SurrealMCP, Agno, and On-Prem Ollama for a SaltStack AI Agent

## Overview

This document outlines a complete, on-premises setup for building an AI-powered Master Agent that integrates SurrealDB (via SurrealMCP) for persistent memory, Agno as a Python-based multi-agent framework for orchestration, and Ollama for local LLM inference. This replaces custom SurrealDB integrations in your current Python + LangChain setup, addressing issues like parsing, table creation, and reliability.

The setup focuses on:
- **SurrealMCP**: Official MCP server for exposing SurrealDB as standardized tools (e.g., Query, Insert, Upsert) to your agent.
- **Agno**: Lightweight Python framework to connect your local Ollama model with SurrealMCP tools, enabling reliable memory operations.
- **Ollama**: Your on-prem LLM (e.g., Qwen2.5-Coder) for reasoning and tool calling.
- **Podman Integration**: All containerized components use Podman (with containerd runtime) instead of Docker for better security and rootless operation.

This architecture keeps everything local, fixes LLM hallucinations via forced tool calls, and paves the way for full SaltStack control (e.g., `check_infrastructure`, `accept_minion`). Once set up, you can extend it toward auto-provisioning minions and deploying trading bots via natural language prompts.

**Assumptions**:
- You're on a Linux host (e.g., Ubuntu/RHEL) with Podman installed.
- SurrealDB is already running on-prem (e.g., at `ws://localhost:8000/rpc`).
- You have Python 3.10+ for Agno.
- Target LLM: Qwen2.5-Coder (adjust as needed; prefer models with strong tool-calling support).

## Prerequisites

1. **Install Podman and Containerd**:
   - Podman is a daemonless container engine. Use containerd as the runtime for better performance and compatibility.
   - On Ubuntu/Debian:
     ```
     sudo apt update
     sudo apt install podman containerd
     ```
   - On RHEL/Fedora:
     ```
     sudo dnf install podman containerd
     ```
   - Configure Podman to use containerd runtime (edit `/etc/containers/containers.conf` if needed, but defaults often work).
   - Verify: `podman --version` and `podman info | grep runtime`.

2. **Install Ollama**:
   - Download and install Ollama binary directly (non-containerized for simplicity, but container option below).
     ```
     curl -fsSL https://ollama.com/install.sh | sh
     ```
   - Pull your model: `ollama pull qwen2.5-coder:14b` (or your preferred variant).
   - Alternative: Run Ollama in Podman (rootless):
     ```
     podman run -d --name ollama -p 11434:11434 -v ollama:/root/.ollama --runtime=containerd docker.io/ollama/ollama
     podman exec ollama ollama pull qwen2.5-coder:14b
     ```
     - Access at `http://localhost:11434`.

3. **Install SurrealDB** (if not already running):
   - Run in Podman:
     ```
     podman run -d --name surrealdb -p 8000:8000 --runtime=containerd docker.io/surrealdb/surrealdb:latest start --user root --pass root
     ```
     - Connect via `ws://localhost:8000/rpc` with NS=agent, DB=salt (adjust as per your project).

4. **Install Python Dependencies for Agno**:
   - Create a virtual env:
     ```
     python -m venv agno-env
     source agno-env/bin/activate
     ```
   - Install Agno and dependencies:
     ```
     pip install agno langchain-ollama  # Agno for agents, langchain-ollama for Ollama integration if needed
     ```

## Step-by-Step Setup

### 1. Run SurrealMCP
SurrealMCP exposes SurrealDB as MCP tools. Run it in Podman for isolation.

- Pull and run:
  ```
  podman run -d --name surrealmcp -p 8080:8080 --runtime=containerd \
    -e SURREALDB_URL=ws://host.containers.internal:8000/rpc \
    -e SURREALDB_NS=agent \
    -e SURREALDB_DB=salt \
    -e SURREALDB_USER=root \
    -e SURREALDB_PASS=root \
    docker.io/surrealdb/surrealmcp:latest start --dev
  ```
  - Notes:
    - `--dev`: Disables auth for local testing (enable JWT for production).
    - Use `host.containers.internal` to connect to host's SurrealDB from the container.
    - Access MCP endpoint at `http://localhost:8080` (or WebSocket for real-time).
  - Verify: `curl http://localhost:8080/health` should return OK.

### 2. Configure Agno with Ollama and SurrealMCP
Agno acts as the agent "brain," connecting your local Ollama model to SurrealMCP tools.

- Create a Python script (`agent.py`) based on the official SurrealDB Agno tutorial:
  ```python
  from agno.agents import Agent
  from agno.models.ollama import Ollama
  from agno.tools.mcp import MCPTools

  # Connect to local Ollama
  ollama_model = Ollama(id="qwen2.5-coder:14b", base_url="http://localhost:11434")

  # Connect to SurrealMCP (MCP tools for SurrealDB)
  mcp_tools = MCPTools(
      endpoint="http://localhost:8080",  # Your SurrealMCP
      auth=None  # None for --dev mode; use JWT for prod
  )

  # Create the agent
  agent = Agent(
      model=ollama_model,
      tools=[mcp_tools],  # Add your SaltStack tools here later
      system_prompt="""You are a SaltStack Master Agent. Use MCP tools for memory (e.g., Insert for history, Query for recall).
      Always call check_infrastructure before status answers to avoid hallucinations.
      Store all actions in SurrealDB under table 'history'."""
  )

  # Interactive chat loop (replace with your existing interface)
  while True:
      prompt = input("Enter prompt: ")
      response = agent.invoke(prompt)
      print(response)
  ```

- Run it: `python agent.py`
- Test: Prompt like "Store this test in history" – it should use MCP Insert tool.
- For history recall: Add prompts like "Recall recent history" – agent queries via MCP Select/Query.

### 3. Integrate SaltStack Tools
Extend Agno to include your SaltStack tools (e.g., from your current Python setup).

- Define custom tools in Agno (or expose as another MCP server later for purity).
  ```python
  from agno.tools import Tool

  # Example: check_infrastructure tool
  def check_infrastructure(args):
      # Your existing SaltStack logic (e.g., salt '*' test.ping)
      import salt.client
      client = salt.client.LocalClient()
      return client.cmd('*', 'test.ping')  # Returns dict of minion status

  salt_tools = [
      Tool(
          name="check_infrastructure",
          description="Check SaltStack cluster health and minion count.",
          function=check_infrastructure,
          schema={}  # Define args if needed
      ),
      # Add accept_minion, apply_salt_state, etc.
  ]

  # Add to agent
  agent = Agent(
      model=ollama_model,
      tools=[mcp_tools] + salt_tools,
      ...
  )
  ```
- This allows natural language prompts like "Check infrastructure status" to call tools and store results in SurrealDB via MCP.

### 4. Enhance for Self-Awareness and Trading Bots
- **Auto-Provisioning**: Use Salt Reactor in your SaltStack master. Trigger agent actions via webhooks or event listeners (e.g., Python script watching Salt events, invoking Agno agent).
- **Trading Bots Example**: Prompt: "Create 3 simulation trading bots for Polymarket."
  - Agent reasons: Breaks into steps (accept minions, apply states).
  - Uses Salt tools to deploy (e.g., apply_salt_state for bot configs).
  - Stores history: MCP Insert for audit.

- For forced tool calling: In system_prompt, enforce "Always call check_infrastructure first for status."

## Troubleshooting and Best Practices
- **Hallucinations**: Stick to strict prompts; test models like Qwen3 for better tool adherence.
- **Podman Issues**: If networking fails, use `--network=host`. For rootless, ensure user namespaces are enabled.
- **Scaling**: Run multiple Podman containers (e.g., one for SurrealDB, one for MCP, Ollama native).
- **Security**: Switch SurrealMCP to JWT auth; restrict Podman to user mode.
- **Testing**: Start with simple prompts. Monitor Ollama logs for tool calls.
- **Resources**:
  - SurrealDB Agno Tutorial: [Official Blog](https://surrealdb.com/blog/building-multi-tool-agents-with-surrealmcp-and-agno)
  - Agno Docs: [GitHub](https://github.com/agnoai/agno)
  - SurrealMCP Repo: [GitHub](https://github.com/surrealdb/surrealmcp)
  - Podman Docs: [Official Site](https://podman.io/docs)

This setup should integrate seamlessly with your Antigravity IDE – copy-paste into a new project, adjust paths/ports, and execute the scripts/commands to bootstrap. If you need expansions (e.g., full Rust migration), let me know!