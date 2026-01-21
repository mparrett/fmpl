# Gas Town & Loom: Relevance to Collaborative AI

**Date**: 2026-01-20

---

## Gas Town Overview

**Project**: Multi-agent workspace manager for Claude Code
- URL: https://github.com/steveyegge/gastown
- **Purpose**: Coordinate multiple Claude Code agents working on different tasks
- **Key innovation**: Git-backed hooks for persistent work tracking
- **Scale**: Designed for 20-30 agents comfortably

### Problem Solved

**Challenge**: Agents lose context on restart
- Manual agent coordination breaks state
- Work disappears when agents crash
- 4-10 agents become chaotic

**Gas Town Solution**:
- Work state persisted in **Beads** (git worktree items)
- Agents read/write beads via mailboxes
- **Propulsion mechanism**: Git hooks as version control
- Scale to 20-30 agents without chaos

### Architecture

```
┌─────────────┐      ┌─────────────┐
│  Mayor 🎩   │      │  Town 🏘️   │
│  AI Coordinator │      │  Workspace   │
└─────────────┘      └─────────────┘
      │                     │
      ├───────┐             │
      ↓         ┌─────────┴─────────┐
   ┌──────────────┐  │  ┌──────────────┐  │
   │ Rig 1 │ Rig 2 │  │  │  Crew 1 │ Crew 2 │  │
   │ (proj A) │ (proj B) │  │  │  (user A) │ (user B) │  │
   └─────┬─────┘  └─────┬─────┘  └─────┬─────────┘
         │              │              │              │
         ├──────┐              └──────┐       ┌────┐
         ↓              ┌─────────────┴─────────┐       ↓         │
   ┌──────────────┐  │  ┌──────────────┐  │  ┌──────────────┐
   │ Hooks 1     │  │  │ Hooks 2     │  │  │  Beads     │  │
   │ (git worktree)│  │  │ (git worktree)│  │  │  (git repo)  │  │
   └─────┬───────┘  └─────┬───────┘  └─────┬─────────┘
         │              │              │              │
         ├──────────────────────────────┐   └──────────────────────────────┘
         │
         ↓
   ┌─────────────────────────────────┐
   │        Git Repository        │
   │      (Beads Ledger)        │
   └─────────────────────────────────┘
```

### Key Components

**Mayor** (AI Coordinator)
- Claude Code instance with full context
- Analyzes tasks, breaks down into convoys
- Spawns agents, assigns beads

**Town** (Workspace)
- `~/gt/` directory contains all rigs/crews
- Multi-user support (each user has crew)

**Rigs** (Project Containers)
- Wrap git repositories
- Associated crew members per rig

**Crew Members** (Personal Workspace)
- Your work area within a rig
- Where you do hands-on development

**Polecats** (Worker Agents)
- Spawned by Mayor, assigned beads
- Execute work, report completion

**Hooks** (Persistent Storage)
- Git worktree-backed ledger
- Beads represent work items
- Propulsion: Git hooks = version control

**Convoys** (Work Tracking)
- Bundle multiple beads (work items)
- Assigned to agents via mailboxes
- Progress tracking via `gt convoy` commands

**Beads Integration**
- Git-backed issue tracking
- Bead IDs: prefix + 5-character alphanumeric (`gt-abc12`)
- Terms "bead" and "issue" used interchangeably

---

## Loom Overview (Revisited)

**Project**: AI-powered coding agent built in Rust
- URL: https://github.com/ghuntley/loom
- **Focus**: Modular agent framework with server-side LLM proxy
- **Status**: 849 stars, research/experimental

### Architecture

```
┌─────────────┐
│  loom-cli   │
└─────────────┘
      │
      ┌──────────────┐
      │  loom-server  │
      └──────────────┘
              │
      ┌──────────────┐
      │  LLM APIs      │
      │  (OpenAI,     │
      │  Anthropic,    │
      │  etc.)         │
      └──────────────┘
```

**Key Components**:
- **30+ crates** for modularity
- **Server-side LLM proxy** (API keys never leave server)
- **Provider-agnostic** (trait-based integrations)
- **Weaver** for remote execution (K8s pods)
- **Thread system** for conversation persistence

---

## Comparison: Gas Town vs. Collaborative AI Vision

| Aspect | Gas Town | Loom | Collaborative AI Vision |
|--------|----------|-------|---------------------|
| **Coordination model** | Git-backed beads (mailboxes) | LLM proxy + CLI | Tuple space + reactive scheduler |
| **Agent persistence** | Beads in git worktree | Thread system | Tuple store (Fjall) |
| **State recovery** | Git hooks (version control) | Thread persistence | Tuple replay + subscriptions |
| **Scaling** | 20-30 agents | Unlimited (server resources) | N workers (horizontal) |
| **Multi-agent** | ✅ Core feature | ❌ Single agent focus | ✅ Core feature |
| **LLM integration** | Claude Code specific | Multi-provider traits | Centralized client |
| **Human-in-the-loop** | Implicit (mailboxes) | Tool-based contact | Modal prompts, escalation |
| **Security** | Git access control | Server-side key storage | PASETO capabilities |
| **Architecture** | Git + CLI (Go) | Rust traits | Coordinator + workers |

---

## Key Insights

### What Gas Town Does Well

**Git-Based Persistence**
- **Beads as work items**: Durable, version-controlled
- **Propulsion**: Git hooks provide audit trail
- **Recovery**: Work survives crashes, restarts
- **Multi-agent**: 20-30 agents coordinated through convoys

**Propulsion Principle** (unique to Gas Town)
- Git hooks drive state machine
- Each bead = work item, git commit = state transition
- Agents consume beads, produce result beads
- Automatic recovery via `git rebase`

### What Loom Does Well

**Modular Architecture**
- **30+ crates**: Clean separation of concerns
- **Trait-based providers**: Easy to add new LLM APIs
- **Server-side keys**: API security (never leave server)

**But Missing for Collaborative Vision**
- ❌ No tuple space coordination
- ❌ No reactive scheduling
- ❌ No multi-user architecture
- ❌ No capability-based security
- ❌ No task lifecycle (spawn/fork/suspend)

---

## Synthesis: Mapping to Collaborative AI Vision

### Gas Town Contributions

| Vision Component | Gas Town Feature | Alignment |
|-----------------|-----------------|-----------|
| **Task abstraction** | Beads (work items) | 🔄 Map to tuples |
| **Reactive dispatch** | Convoys (mailboxes) | 🔄 Add subscriptions |
| **Multi-agent coordination** | Mayor assigns to convoys | 🔄 Scheduler + tuple space |
| **State persistence** | Git hooks (beads) | 🔄 Fjall tuple store |
| **Human-in-the-loop** | Mailboxes | 🔄 Modal prompts, capability escalation |
| **Scalability** | 20-30 agents | 🔄 Unlimited (N workers) |
| **Version control** | Git propulsion | 🔄 Durable tuple replay |

### Loom Contributions

| Vision Component | Loom Feature | Alignment |
|-----------------|-----------------|-----------|
| **Modular architecture** | 30+ crates, traits | ✅ Fully aligned |
| **LLM integration** | Server-side proxy, multi-provider | ✅ Centralized client approach |
| **Tool orchestration** | Tool registry + execution | 🔄 Map to tuple space dispatch |
| **State management** | Thread system | 🔄 Map to tuple store |
| **Remote execution** | Weaver (K8s) | 🔄 Worker sandboxing |
| **Security** | Server-side keys | 🔄 Add PASETO layer |

### Missing from Both

| Missing | Why It Matters |
|---------|---------------|
| **Tuple space** | Neither has Linda-style coordination | Enables data-centric coordination, no addressing |
| **Reactive scheduling** | Neither has subscriptions | Eliminates polling, push-based |
| **Capability security** | Gas Town: git ACL, Loom: server keys | PASETO + hierarchical attenuation |
| **Task lifecycle** | Gas Town: beads, Loom: not explicit | First-class tasks, spawn/fork/suspend |

---

## Recommendations for Collaborative AI Vision

### 1. Adopt Tuple Space + Reactive Scheduling

Both Gas Town and Loom lack this core primitive:
- **Linda-style `out`/`in`/`rd` operations**
- **Pattern-based subscriptions** (not polling)
- **Time/space decoupling** (agents don't need to know each other)

### 2. Learn from Gas Town's Propulsion Model

**Git hooks as state machine propulsion**:
- Beads = work items (map to tuples)
- Git commits = state transitions (map to task status updates)
- Hooks = triggers (map to reactive subscriptions)
- Automatic versioning = built-in audit trail

**Why this works**:
- Durable, recoverable
- Debuggable (git log shows all transitions)
- Decentralized (each agent reads/writes to git)

### 3. Modular Provider Architecture (Loom Pattern)

**Trait-based LLM providers**:
```rust
trait LlmProvider {
    async fn complete(&self, prompt: &str) -> Result;
    async fn stream(&self, prompt: &str) -> Stream;
}

// Multiple implementations
struct AnthropicProvider;
struct OpenAIProvider;
struct ClaudeProvider;
```

**Benefits**:
- Provider-agnostic coordinator
- Easy to add new providers
- Cost tracking per provider
- Per-org rate limiting

### 4. Human-in-the-Loop Design (Gas Town Approach)

**Mailboxes as explicit contact**:
- Agent asks for approval: `gt sling gt-abc12 --notify`
- Human sees all pending requests in one place
- Bead IDs provide traceability
- **No polling** - git hooks trigger when beads change

**Combine with Loom's structured outputs**:
- `gt sling` = LLM tool call
- Structured JSON responses
- Tool registry for validation

### 5. Agent Capability Model

**Gas Town**: Git access control (rig-level)
- Each rig has its own git repository
- ACLs on who can access which rig

**Vision addition**: PASETO tokens
- Hierarchical: User → Session → Agent → Task
- Server-side verification (Loom already does this)
- Instant revocation (Gas Town: revoke git access, Loom: delete signing key)

### 6. RLM Patterns (Partition + Map)

**Neither implements explicitly**, but both enable foundation:

**Gas Town**:
- Convoys bundle multiple beads
- Mayor breaks work into tasks
- Parallel agents work on different convoys

**Loom**:
- Thread system for conversations
- Server-side LLM proxy
- Modular tool registry

**Missing**: Query-based context sharing
- Partition+map: "spawn N subtasks for different parts of codebase"
- Summarize results to tuple store: `insight tid-123 summary "..."`
- Other agents query before re-reading files

---

## Conclusion

**Gas Town**: Excellent multi-agent coordination with git-based persistence, but missing tuple space and reactive scheduling

**Loom**: Excellent modular architecture with server-side LLM proxy, but single-agent focus, no coordination layer

**Your Vision**: Extends both by adding:
- Tuple space coordination (Linda-style + reactive subscriptions)
- Capability-based security (PASETO + hierarchical attenuation)
- Task lifecycle (spawn/fork/suspend/resume)
- RLM patterns (query-based context, partition+map)

**Strategic Recommendation**:
- Use Gas Town's propulsion model (git hooks) for state machine
- Use Loom's modular architecture (trait-based providers)
- Add tuple space (neither has this)
- Add reactive scheduler (neither has this)
- Learn from 12 Factor Agents (small/focused, stateless reducers)

**Status**: Research complete. Ready to inform design decisions.

---

**Next:**
- Incorporate Gas Town propulsion into tuple space design?
- Learn from Beads formula system for repeatable workflows?
- Study Loom's 30-crate architecture for modular coordinator?
