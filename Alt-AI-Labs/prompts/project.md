# SaltStack + LLM Master Agent Project Summary

**Last Updated:** March 29, 2026

## What Are We Doing?

We are building an **AI-powered Master Agent** that fully controls a SaltStack cluster using natural language prompts.

The ultimate goal is to create a **self-aware, agentic infrastructure system** where:

- You give high-level prompts (e.g. “Create 3 simulation trading bots for Polymarket”, “Find and complete a Fiverr gig”, or “Deploy a new minion”).
- The Master Agent uses a local LLM to reason and plan.
- It discovers and calls standardized tools via the **Model Context Protocol (MCP)**.
- The agent automatically detects new minions, provisions them, applies states, runs commands, and stores every action for audit and self-improvement.
- All cluster state, history, reflections, and gig outcomes live in SurrealDB for long-term memory.

This combines traditional DevOps (SaltStack) with modern agentic AI workflows, now fully realized as a **pure-Rust** solution.

## Current Architecture

**Hardware Layout and software(as of March 29, 2026):**
- **Mac Air** 
    - Runs Ansible to provision master and minion Ubuntu servers
    - Development environment
        - Antigravity
        - Cursor
    - Master of master
- **Alt AI Master** 
    - **SaltMaster** (with Salt REST API enabled) + **2 connected Minion nodes** (bare Salt minions).
    - **SurrealDB** — version 3.x
        - **SurrealMCP** 
    - **saltapiMCP** 
    - **githubMCP**
    - **xaiMCP**
    - **ollama** with 2060 GPU

**Key Design Decision:**  
We deliberately created a **standalone `saltapimcp` and `xapimcp`** Rust projects instead of embedding logic inside the agent. This mirrors SurrealMCP and gives us clean tool discovery, reusability, and the ability to run saltapimcp, xapimcp anywhere (even on the GPU server itself).

## Approach We Are Taking

We are using a **pure-Rust, MCP-first architecture**:

- **MCP everywhere** — SurrealMCP for memory + new `saltapimcp` for infrastructure. The agent only speaks MCP.
- **Rig + rmcp** — Rust-native agent framework with excellent Ollama and MCP support (forced tool calling, ReAct loops, type-safe schemas).
- **Standalone `saltapimcp`** — A separate Rust binary/service that wraps the Salt REST API and will later listen to Salt events/reactors for true self-awareness.
- **Podman + containerd** — Only `gitlabMCP` will run in a container.  All other services (SurrealDB, SurrealMCP, saltapimcp, xapimcp) run as a service.

We have completely moved away from Python/LangChain/custom SurrealDB glue code.

## Where We Are in the Process

**Completed:**
- SurrealBDMCP - (compiled and tested)
- saltapiMCP - (compiled and tested)
- xapiMCP - (compiled)
- service files for githubMCP - Service file is ready

**In Progress / Next (immediate):**
- configuring `githubMCP` and `xapimcp`
    - create github access token
    - create X API account
- Build the thin `salt-master-agent` binary that connects to both SurrealMCP and `saltapimcp`.
- Replace all previous memory/history logic with SurrealMCP tools.
- Add forced tool-calling pattern and strict system prompts to eliminate hallucinations.

**Upcoming:**
- Implement Salt Reactor logic inside `saltapimcp` for automatic minion detection and provisioning.
- Retrieve my X Bookmarks and store them in SurrealDB
    - schedule this pull
    - only pull number allowed by X api
- Expand toward autonomous project execution 
    - X Bookmark (If bookmark talks about a github project, make a plan to test it)
    - Fiverr/Upwork monitoring → provision → code → submit.
- Containerize `saltapimcp` with Podman and run it alongside SurrealMCP.

## Challenges We Have Faced (and Solved)

1. **LLM Hallucination** — Solved by moving to strict MCP tool schemas in Rust + forced tool calling in Rig.
2. **Tool Calling Reliability** — Local Ollama models now get clean, discoverable tools via MCP instead of fragile LangChain bindings.
3. **SurrealDB Integration** — Completely eliminated custom code by adopting official SurrealMCP.
4. **Python Caching & Fragility** — Removed entirely by going full Rust (single static binary, no venv hell).
5. **Prompt Engineering** — Now handled via typed tool definitions and clean system prompts.
6. **Long Debugging Cycles** — Greatly reduced with Rust’s compile-time safety and MCP standardization.


---

This rewritten summary reflects every decision we’ve made across our conversations. You can copy-paste it directly into your repo or Antigravity IDE as the new living document. Let me know if you want the Mermaid diagrams updated to match this Rust/MCP architecture as well!