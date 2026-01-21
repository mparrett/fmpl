# Loom & Gatsby Orchestrations: Relevance to Collaborative AI

**Date**: 2026-01-20

---

## These Projects

**Geoffrey Huntley's Loom** (https://github.com/ghuntley/loom)
- AI-powered coding agent built in Rust
- REPL interface for LLM-powered agents
- Modular architecture (30+ crates)
- Server-side LLM proxy (API keys never leave server)

**Stevey Yegge's Gatsby Orchestrations** (https://github.com/steveyegge/gatsby-orchestrations)
- Content/workflow orchestration for Gatsby
- Multi-repo coordination

---

## Key Architectural Patterns

### Loom's Core Principles

From project overview (reconstructed):

**Modular Architecture**
```
loom/
├── crates/
│   ├── loom-core/           # Core abstractions, state machine, types
│   ├── loom-server/         # HTTP API server with LLM proxy
│   ├── loom-cli/            # Command-line interface
│   ├── loom-thread/         # Conversation persistence and sync
│   ├── loom-tools/          # Agent tool implementations
│   ├── loom-llm-*/          # LLM provider integrations
│   └── ...
├── web/
│   └── loom-web/            # Svelte 5 web frontend
├── specs/                   # Design specifications
└── infra/                   # Nix/K8s infrastructure
```

**Server-Side LLM Proxy**
```
┌─────────────┐      HTTP       ┌─────────────┐
│  loom-cli   │ ───────────────▶│ loom-server │ ──────────────────▶ │  LLM APIs
│             │ /proxy/{provider}│             │                     │
│ ProxyLlm    │  /stream        │  LlmService │                     │
│ Client      │  /complete      │             │                     │
└─────────────┘ ◀─────────────────────────────── └─────────────────┘
```

- API keys stored server-side only
- LLM provider-agnostic (traits + implementations)
- SSE streaming for real-time responses

**Weaver Remote Execution**
- Kubernetes pods for isolation
- Remote execution environments

**Analytics**
- PostHog-style product analytics
- Identity resolution

---

## Alignment with 12 Factor Agents

**What Loom does well (from 12 Factor Agents perspective):**

✅ **Factor 1: Natural Language to Tool Calls**
- Loom's tool system provides structured outputs from LLM
- Clean separation between LLM decisions and deterministic code execution

✅ **Factor 2: Own Your Prompts**
- Modular prompt management (not black-box framework)
- Full control over LLM interactions

✅ **Factor 3: Own Your Context Window**
- Custom context formats optimized for token efficiency
- Thread system for conversation persistence

✅ **Factor 4: Tools Are Just Structured Outputs**
- Registry and execution framework for agent capabilities
- Deterministic code triggers on structured JSON from LLM

✅ **Factor 5: Unify Execution State and Business State**
- Thread system unifies state (simple serialization)
- No complex abstractions between execution and business state

✅ **Factor 7: Contact Humans with Tool Calls**
- Human-in-the-loop support
- Tools for different types of human contact (approval, clarification)

✅ **Factor 8: Own Your Control Flow**
- Custom control structures
- Interrupt and resume at any point

✅ **Factor 11: Trigger from Anywhere**
- Webhook support for external triggers
- Enable outer-loop agents (cron, events)

✅ **Factor 12: Stateless Reducers**
- Thread system allows simple stateless agent implementations
- Each agent is just handling events and deciding next action

---

## Relevance to Collaborative AI Vision

### What Loom Provides

| Aspect | Loom | Collaborative AI Vision |
|--------|------|----------------------|
| **Agent execution** | REPL + tools in Rust | DSL runtime + tuple store |
| **LLM integration** | Server-side proxy | Centralized LLM client |
| **Multi-agent** | Multi-provider (traits) | Tuple space coordination |
| **Security** | Not specified | PASETO capabilities |
| **Persistence** | Thread system | Fjall tuple store |
| **Human-in-the-loop** | Tool-based contact | Modal prompts, pause/resume |
| **Scalability** | Kubernetes (Weaver) | Workers + coordinator |
| **Modularity** | 30+ crates, trait-based | Coordinator + workers |

### Key Convergence Points

**Both address:**

1. **Tool orchestration**
   - Loom: Registry and execution framework
   - Vision: Tuple store with reactive subscriptions

2. **Multi-provider LLM support**
   - Loom: Trait-based, pluggable providers
   - Vision: Centralized client (easier cost tracking, caching)

3. **Human-in-the-loop**
   - Loom: Structured tool outputs for different contact types
   - Vision: Modal prompts, capability escalation

4. **Durable state**
   - Loom: Thread system for conversation persistence
   - Vision: Tuple store write-through to Fjall

5. **Scalable architecture**
   - Loom: Server + workers (Weaver K8s)
   - Vision: Coordinator + workers (N sandboxed processes)

---

## What Loom Lacks (Vision Has)

| Aspect | Loom | Collaborative AI Vision |
|--------|------|----------------------|
| **Tuple space** | ❌ No Linda-style coordination | ✅ `out`/`in`/`rd` + subscriptions |
| **Reactive scheduling** | ❌ No subscription model | ✅ Reactive dispatch on tuple changes |
| **Capability security** | ❌ No PASETO/attenuation | ✅ Hierarchical capabilities |
| **Namespace isolation** | ❌ Not specified | ✅ `/user/`, `/project/` isolation |
| **Task lifecycle** | ❌ No spawn/fork/suspend | ✅ Full task model |
| **RLM patterns** | ❌ No partition+map | ✅ Query-based context sharing |

**Loom is closer to "many small agents" approach** (12 Factor #10), while vision is "coordination platform for RLM workflows" (combines 1-12).

---

## Gatsby Orchestrations (Inferred)

From project name, this appears to be:
- Multi-repo coordination for Gatsby builds/content
- Workflow orchestration across codebases
- CI/CD automation integration

**Relevance**: Demonstrates orchestration patterns, but different domain (content vs. AI coding). Less directly relevant to AI agent coordination.

---

## Synthesis: How These Inform Vision

### Architectural Insights

**1. Modularity is Proven**
- Loom's 30+ crates show trait-based extensibility works
- Vision's coordinator + workers achieves similar modularity
- Both avoid monolithic "everything in one process" design

**2. Server-Side LLM Keys Work**
- Loom demonstrates API keys never need to leave server
- Vision's centralized LLM client provides same benefit
- Both enable cost tracking, rate limiting per organization

**3. Tool Registry Pattern**
- Loom: Structured tool outputs → deterministic code
- Vision: Tuple space patterns → reactive dispatch
- Both separate LLM decision-making from execution

**4. Human-in-the-Loop is Critical**
- Loom: Tools for different contact types
- Vision: Modal prompts for capability escalation
- Both acknowledge agents need human approval for high-stakes actions

**5. State Management Matters**
- Loom: Thread system unifies execution + business state
- Vision: Tuple store unifies coordination state
- Both enable pause/resume and crash recovery

---

## Recommendations for Vision

**Learn from Loom:**

1. **Adopt modular, trait-based architecture**
   - Coordinator uses traits for LLM providers
   - Workers use traits for tool sets
   - Easy to add new providers/tools

2. **Implement server-side LLM proxy**
   - API keys stored centrally
   - Provider-agnostic interface
   - Per-org cost tracking, rate limits

3. **Tool-based agent model**
   - Tools = structured outputs from LLM
   - Deterministic code handles tool calls
   - Clean separation of concerns

**Go Beyond Loom (Vision's Novel Contributions):**

4. **Add tuple space with reactive subscriptions**
   - Linda-style `out`/`in`/`rd` operations
   - Pattern matching for coordination
   - Push on change, not polling

5. **Implement capability-based security**
   - PASETO tokens (not JWT)
   - Hierarchical attenuation (User → Session → Task)
   - Instant revocation, audit trails

6. **Add task lifecycle with spawn/fork/suspend**
   - First-class tasks as entities
   - Durable state serialization
   - Commit before fork (no nested transactions)

7. **Add RLM patterns**
   - Partition+map for parallel subtasks
   - Query tuple store instead of restuffing context
   - Summary/insight tuples for knowledge sharing

---

## Conclusion

Loom demonstrates that **modular, tool-based, server-side LLM proxy** architecture works for AI agents. It implements most of **12 Factor Agents** principles.

Your collaborative AI vision extends this by adding:
- **Tuple space coordination** (Linda-style + reactive subscriptions)
- **Capability-based security** (PASETO + hierarchical attenuation)
- **Task lifecycle** (spawn/fork/suspend/resume)
- **RLM workflows** (partition+map, query-based context)

This is a natural evolution: Loom provides the agent framework; your vision provides the coordination layer.

---

**Next Steps:**
- Study Loom's crate structure for implementation patterns
- Incorporate 12 Factor insights into design decisions
- Reference Loom's server-side proxy for centralized LLM management
- Learn from Weaver K8s patterns for worker scaling

**Status**: Ready to deepen research and incorporate insights.
