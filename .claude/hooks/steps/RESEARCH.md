This is the research arc. You need codebase discovery before implementation.

**You MUST use Explore subagents** -- do NOT do serial Read/Grep calls yourself.

1. Dispatch ONE Explore subagent with a detailed prompt describing what you need to learn.
2. Optionally dispatch a second subagent for a different aspect (parallel is fine).
3. Synthesize findings and proceed to DOCUMENT (write to docs/codebase/).

Rules:
- Max 2 subagent dispatches. If you need more, your prompt wasn't specific enough.
- Direct Read/Grep only for specific files the subagent identified.
- Write/Edit transitions to IMPLEMENT (if you decide to code instead).
