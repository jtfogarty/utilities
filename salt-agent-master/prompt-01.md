# SaltStack + LLM Master Agent Project Summary

**Last Updated:** March 16, 2026

## What Are We Doing?

We are building an **AI-powered Master Agent** that controls a SaltStack cluster using natural language prompts.

The ultimate goal is to create a **self-aware, agentic infrastructure system** where:

- You give high-level prompts (e.g. "Create 3 simulation trading bots for Polymarket" or "Deploy a new minion").
- The Master Agent uses an LLM to reason.
- It uses tools to interact with SaltStack (check status, accept minions, apply states, run commands).
- New minions are automatically detected and provisioned (self-aware behavior).
- All actions and history are stored in SurrealDB for memory and audit.

This combines traditional DevOps (SaltStack) with modern agentic AI workflows.

## Current Architecture

**Components:**
- **SaltStack Master** — Core infrastructure management
- **Master Agent** — LLM-powered brain (Python + LangChain + Ollama)
- **Tools** — `check_infrastructure`, `list_minions`, `accept_minion`, `run_salt_command`, `apply_salt_state`
- **SurrealDB** — Persistent memory and state storage
- **Redis** — Available for future task queuing

**Current Capabilities:**
- Interactive chat interface
- Reliable live health checks (correctly reports 0 minions + HEALTHY)
- Basic memory storage in SurrealDB
- History recall (`history`, `sitrep`, `summary`)

## Approach We Are Taking

We are using a **hybrid agent architecture**:

- **Forced tool calling pattern** — Because local models (especially Qwen2.5-Coder) tend to hallucinate numbers, we force the model to call `check_infrastructure` before answering status questions.
- **ReAct-style reasoning** with strict system prompts.
- **SurrealDB** as the long-term memory store.
- Iterative development: get one thing working reliably before adding complexity.

We deliberately **avoided Kubernetes** as per your original preference, relying on SaltStack as the primary orchestration layer.

## Where We Are in the Process

**Completed:**
- Stable interactive chat interface
- Reliable `check_infrastructure` tool that returns correct data (0 minions, HEALTHY)
- Basic memory storage in SurrealDB
- History recall functionality (though still improving)
- Clean separation between tools and agent logic

**In Progress / Next:**
- Improve memory recall reliability and formatting
- Add support for remaining tools (`accept_minion`, `apply_salt_state`, etc.)
- Build the **Salt Reactor** for self-aware auto-provisioning of new minions
- Expand toward the original goal: deploying trading bots via natural language prompts

## Challenges We Have Faced

1. **LLM Hallucination** — The biggest ongoing issue. Models frequently invent minion counts (10, 15, 50, 95…) even when the tool clearly returns 0.

2. **Tool Calling Reliability** — Local Ollama models have weak/ inconsistent tool calling support. DeepSeek-Coder-V2 (recommended by Karpathy) doesn't support tools at all.

3. **SurrealDB Integration** — Multiple issues with result parsing, table creation, and query structure. Memory saving worked before reading did.

4. **Python Caching** — Old versions of `tools.py` kept being loaded despite edits.

5. **Prompt Engineering** — Required extremely strict, repetitive instructions to prevent the model from guessing numbers.

6. **Long Debugging Cycle** — We went through many iterations of ReAct loops, LangGraph attempts, forced-call patterns, and prompt variations.

---

## Current Status (as of March 16, 2026)

We have a **working interactive Master Agent** that can:
- Answer live status questions correctly
- Store interactions in SurrealDB
- Recall history (with some formatting issues)

We are now ready to expand functionality toward full cluster control and your original trading bot orchestration use case.

---