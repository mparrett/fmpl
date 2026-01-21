# Collaborative Agentic Engineering System: Design Summary

**Vision**: Enable human + AI teams to code together through secure, scalable coordination

## Problem Statement
Modern AI coding assistants operate in isolation:
- No collaboration between AI agents
- No shared state or coordination
- No security boundaries or sandboxing
- No durable pause/resume for long workflows
- No multi-user support

Developers copy-paste between tools, losing context and wasting time.

## Solution: Coordinator + Tuple Space + Workers

**Architecture**: 3-layer coordination system

1. **Coordinator Daemon**: Central hub with tuple store + reactive scheduler + LLM client
2. **Workers**: N sandboxed processes (cgroups, seccomp) executing code
3. **Terminal**: Narrative UI for human-in-the-loop interaction

**Alternative**: Multi-VAT architecture (as implemented in FMPL) without central coordinator — VATs coordinate via tuple space

**Key Innovation**: Tuple space as shared coordination primitive
- Agents read/write tuples (Linda-style: `out`/`in`/`rd`)
- Reactive subscriptions fire on changes (no polling)
- Persistent to Fjall (instant recovery on restart)

## Capability-Based Security

**Model**: Hierarchical PASETO tokens
```
User (fs:**, shell:*)
  ↓ grants subset
Worker (fs:read:/src/**, shell:cargo)
  ↓ LLM attenuates
Task (fs:read,write:src/foo.rs, shell:cargo test)
  ↓ forks
Subtask (fs:read:src/foo.rs, shell:cargo test --no-run)
```

**Benefits**:
- Principle of least privilege (minimum caps per task)
- Instant revocation (delete signing key)
- Namespace-based isolation (`/project/foo/task:123`)
- Audit trail (all cap changes logged)

## Task Lifecycle: Spawn → Running ↔ Suspended → Done

**Features**:
- **Fork**: Parent commits, child sees committed state (no nested transaction complexity)
- **Suspend**: Serialize state to Fjall, release worker, resume later with re-validated caps
- **Kill**: Non-hierarchical (children become orphans)
- **Exception handling**: Cross-frame unwinding, no half-done state visible

## Reactive Scheduling (No Polling)

```
subscribe (task * status "pending") → dispatch to available worker
subscribe (task * waiting-for "user-input *") → on user input, resume
subscribe (task * waiting-for "subtask *") → on child done, check parent
```

**Benefits**:
- Uniformity: Scheduling = tuples + subscriptions
- Debuggability: Query tuples to see why task didn't resume
- Scalability: O(1) dispatch via subscriptions
- Recovery: On restart, re-subscribe to patterns, scheduling resumes

## Agent Coordination: RLM-Inspired (Recursive Language Models)

**Problem**: LLM context rot — stuffing context causes degradation

**Solution**: Agents query tuple store for what they need, don't pass full context

```
agent A reads file → writes summary tuple
agent B needs file context → queries summary tuple (no re-read)
agent A spawns children → each writes results → parent aggregates from tuples
```

**Benefits**:
- Parallelism: Partition + map spawns subtasks
- Knowledge reuse: Summaries/insights persist in tuple store
- Cost efficiency: Cached LLM calls, no redundant context
- Debuggability: All context queries visible as tuple operations

## Why Tuple Space (Not Actors)?

Actors use explicit addressing (send to object ID), require sender + receiver alive.

**Tuple space advantages**:
- Time/space decoupling: Producers/consumers don't need to know each other
- Data-centric coordination: Match on patterns, not addresses
- Shared memory: No message passing overhead
- Reactive: Push on change, not pull-based

**Use cases**:
- Task scheduling (match `(task ?id status:"pending")`)
- Agent coordination (match `(task-insight ?tid ?insight)`)
- Resource allocation (match `(worker ?w status:"idle")`)

## Implementation: 3-Tier Architecture

```
┌─────────────────────────────────────────┐
│         Terminal (User Interface)       │
│    Narrative UI + Inline Editor       │
└─────────────────┬───────────────────────┘
                  │ ZeroMQ + PASETO
┌─────────────────▼───────────────────────┐
│         Coordinator (Daemon)             │
│  • Tuple Store (in-memory)          │
│  • Reactive Scheduler                 │
│  • LLM API Client                   │
│  • PASETO Signing/Verification        │
└────┬────────┬────────┬───────────┘
     │ Fjall  │ workers│
     ↓ write-  ↓ ZeroMQ│
┌────┴──────┐  ↓ + PASETO  (N sandboxed processes)
│   Fjall    │  ┌───────────────────────────────┐
│  (LSM      │  │ Worker 1 │ Worker 2 │ ...  │
│   Store)    │  │ cgroups   │ cgroups   │      │
└─────────────┘  └───────────────────────────────┘
```

## Tech Stack

- **Rust**: Coordinator + workers (performance + safety)
- **Fjall**: LSM-based storage (durable, instant recovery)
- **ZeroMQ**: High-performance messaging (async, ZeroMQ patterns)
- **PASETO**: Token-based auth (avoids JWT pitfalls)
- **Tokio**: Async runtime (per-worker)
- **FlatBuffer/CapnProto**: Zero-copy encoding (DB to client)
- **cgroups/seccomp**: OS-level sandbox (defense in depth)

## Persistence & Recovery

**Tuple store**: Write-through to Fjall on separate thread
- Hot path: In-memory queries (O(1) pattern matching)
- Cold path: Durable log (replay on restart)
- Recovery: Instant (load tuples, re-subscribe to patterns)

**Task state**: Serialize to Fjall on suspend
- Parse state (VM snapshot)
- Bindings + stack
- Resume: Deserialize, re-validate capabilities before continuing

**Resilience**: All activity transactional + durable
- Coordinator crash → replay Fjall, re-subscribe, pending tasks resume
- Worker crash → coordinator spawns new worker, task retries

## Scalability Model

**Coordinator**: O(1) dispatch (reactive subscriptions)
- Doesn't block on task count
- Tuple queries indexed by pattern type

**Workers**: Horizontal scaling (add more processes)
- Each worker: cgroup limits (CPU, memory, I/O)
- Isolated failure: Worker crash doesn't affect others
- No shared state across workers (tuple space is only coordination)

**LLM**: Centralized access (cost tracking, caching)
- Context-cache tuples prevent redundant calls
- Rate limiting per session/user
- Cost enforcement (abort on budget exceed)

## Security Model

**Defense in depth**:
1. **Transport**: ZeroMQ + Curve encryption
2. **Auth**: PASETO tokens (not JWTs)
3. **Runtime**: Capability checks on every primitive
4. **OS**: cgroups (resource limits), seccomp (syscall filtering)
5. **Network**: Namespace isolation (`/user/`, `/project/`)

**Capability hierarchy**:
- User caps → worker ceiling (session setup, user friction)
- Worker caps → task caps (LLM attenuates based on needs)
- Task caps → subtask caps (further narrowing)

**Escalation flow**: Modal prompts, suspend + request
- Task needs cap → coordinator checks worker ceiling
- Within ceiling → prompt user (grant for task/session/deny)
- Exceeds user caps → deny (impossible to grant)

## Multi-User Model

**Users as first-class entities**:
```
(user <id> name <string>)
(user <id> caps <capability-set>)
(user <id> member-of <project-id>)
```

**Namespaces**:
- `/user/<user-id>/` — private, default home
- `/project/<project-id>/` — shared (explicit membership)
- `/system/` — coordinator internals (read-only to users)

**Project sharing**: Roles (owner/editor/viewer) control grantable caps

**Visibility**: Private by default
- User's tasks/tuples not visible to others
- Project namespace tuples visible to members only
- Future: Opt-in presence ("alice is working on feature X")

## Why This System?

**For developers**:
- Collaborative AI coding (not copy-paste between tools)
- Durable long workflows (pause/resume, recovery from crashes)
- Secure sandboxing (agents can't break system)
- Multi-user support (team workflows, code reviews)

**For researchers**:
- Test agent coordination strategies (RLM-inspired)
- Debuggable (query tuples to understand decisions)
- Reproducible (all activity persistent, no hidden state)

**For operations**:
- Horizontal scaling (add workers)
- Resource controls (cgroups, cost limits)
- Observability (tuple queries show system state)
- Recovery (instant restart, no data loss)

## Implementation Roadmap

**Phase 1**: Coordinator + Tuple Store (3 months)
- In-memory tuple store with pattern matching
- Reactive scheduler with subscriptions
- Fjall write-through persistence
- PASETO signing/verification

**Phase 2**: Worker Runtime (2 months)
- Worker process spawning
- cgroup/seccomp sandboxing
- ZeroMQ + PASETO client
- DSL runtime (tick/time limited)

**Phase 3**: LLM Integration (2 months)
- OpenAI API client with caching
- Cost tracking + limits
- Context-cache tuples

**Phase 4**: Multi-User (2 months)
- User authentication (OAuth2/password)
- Namespace-based isolation
- Project membership + roles

**Phase 5**: Terminal UI (1 month)
- Narrative interface (MOO/Jupyter fusion)
- Inline/popout editor
- Agent task visualization

**Total**: 10 months (small team, parallel phases possible)

## Call to Action

**Two paths forward**:

**Path A: Standalone Agentic System**
- Build coordinator + tuple store + workers from scratch
- We need contributors for:
  - Tuple store implementation (datalog queries, pattern matching)
  - Reactive scheduler (subscription semantics, edge triggers)
  - Worker sandboxing (cgroups, seccomp integration)
  - LLM client (caching, cost controls, streaming)
  - Terminal UI (narrative interface, human-in-the-loop)
  - DSL design (tuple ops + task primitives)

**Path B: Extend FMPL (Multi-VAT)**
- FMPL already has: streaming grammars, facets, async infrastructure
- Add: Tuple space integration, multi-VAT coordination, multi-user
- Consider renaming: Language diverged from original FMPL — deserves new name

**Status**: Both paths viable. Which direction?

**Join us in building** collaborative, secure, scalable AI coding teams where humans and agents work together through tuple space coordination and capability-based security.

---

**Status**: Design complete, ready for implementation. See full spec: `system-spec.md`
