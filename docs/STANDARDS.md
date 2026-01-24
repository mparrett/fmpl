# Documentation Standards

This document defines the standard structure and format for FMPL documentation to ensure consistency across design documents, implementation plans, and specifications.

---

## Design Documents

Design documents (`docs/design/*.md`) describe high-level architectural decisions, technical concepts, and vision for the FMPL project. They are **informational** and do not include implementation tasks.

### Standard Structure

```markdown
# [Descriptive Title]

## Overview
Brief description (1-2 sentences) of what this document covers.

## Core Concepts / The Problem
Explain the motivation, background, and problem being solved.

## Design / Solution
Detailed technical description of the proposed solution or architecture.

### Subsection 1
### Subsection 2
...

## Key Components
List and describe major components if applicable.

| Component | Description |
|-----------|-------------|
| Name | Brief description |

## Examples
Worked examples demonstrating the concept (code snippets, diagrams, etc.)

## Implementation Status
(Optional) Track implementation progress:
- **Complete**: Implemented features
- **In Progress**: Currently being worked on
- **Planned**: Future work

## References
Links to related documents, external resources, etc.
```

### Style Guidelines

- Use present tense for descriptions ("This component handles...")
- Use code blocks for all examples with language specified (`fmpl`, `rust`)
- Include anchor links for major sections
- Keep sections focused and scannable
- Use tables for structured data (comparisons, component lists)

---

## Implementation Plans

Implementation plans (`docs/plans/*implementation*.md`) provide detailed, step-by-step tasks for implementing a feature. They are **actionable** and include acceptance criteria.

### Standard Structure

```markdown
# [Feature Name] Implementation Plan

**Status**: [Draft | In Progress | Complete]
**Date**: YYYY-MM-DD
**Related**: [Link to related design documents](path/to/doc.md)

---

## Overview

Brief description of the feature being implemented.

---

## Goals

1. **Goal 1**: Description
2. **Goal 2**: Description
...

---

## Non-Goals (Out of Scope)

- Feature not being implemented (future work)
- Another out-of-scope item
...

---

## Phase 1: [Phase Name]

### Task 1.1: [Task Name]

**File**: `path/to/file.rs`

**Description**:
Brief description of what this task does.

**Implementation**:
```rust
// Code snippets showing the change
```

**Acceptance criteria**:
- [ ] Criteria 1
- [ ] Criteria 2
- [ ] Criteria 3

---

### Task 1.2: [Task Name]

... (continue for all tasks)

---

## Phase 2: [Future Phase Name]

(For future phases, keep high-level)

### Task 2.1: [Task Name]

...

---

## Testing Strategy

### Unit Tests
- `path/to/test.rs`: Description of test coverage

### Integration Tests
- `tests/integration.rs`: Description

### REPL Tests
```fmpl
# Example REPL test session
let result = some_function()
assert(result == expected)
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Risk description | Mitigation strategy |

---

## Open Questions

1. **Question**: Description
   - **Decision**: Answer or approach

---

## Implementation Order

1. Task 1.1: Description
2. Task 1.2: Description
...

---

## References

- [Related Document](path/to/doc.md)
- [External Resource](url)
```

### Style Guidelines for Implementation Plans

- Use checkboxes for acceptance criteria
- Include file paths for all code changes
- Provide code snippets for complex changes
- Group related tasks into phases
- Mark completed tasks with `[x]`
- Keep tasks atomic and independently verifiable

---

## Specification Documents

Specification documents (`specs/*.md`) describe language features, APIs, or system behaviors. They serve as **reference documentation** for implementers and users.

### Standard Structure

```markdown
# [Feature/Component Name] Specification

## Overview
Brief description of what this spec covers.

## Motivation
Why this feature exists or what problem it solves.

## Syntax / API
The user-facing syntax or API description.

## Semantics
Detailed behavior description.

### Rules / Behaviors
1. **Rule 1**: Description
2. **Rule 2**: Description

## Examples

### Basic Usage
```fmpl
// Example code
```

### Advanced Usage
```fmpl
// More complex example
```

## Edge Cases
Document important edge cases and their handling.

## Implementation Notes
(Optional) Notes for implementers.

## Related Specifications
Links to related specs.

## References
External references, inspiration sources, etc.
```

---

## Common Elements

### Date Format
Use ISO 8601: `YYYY-MM-DD`

### Status Values
- **Draft**: Initial proposal, not yet reviewed
- **In Progress**: actively being implemented
- **Complete**: Implementation finished and verified
- **Deprecated**: No longer recommended

### Link Format
Use relative paths for internal links:
```markdown
[Link text](../../docs/design/document.md)
```

### Code Blocks
Always specify language:
````markdown
```rust
fn example() {}
```

```fmpl
let x = 42
```
````

### Task Format
For implementation tasks, use this format:

```
### Task X.Y: [Descriptive Name]

**File**: `path/to/file`

**Description**:
What and why.

**Implementation**:
```rust
// Code if applicable
```

**Acceptance**:
- [ ] Criterion 1
- [ ] Criterion 2
```

---

## File Naming Conventions

### Design Documents
- Format: `kebab-case.md`
- Examples: `language-guide.md`, `indexed-rpn.md`, `project-overview-draft.md`

### Implementation Plans
- Format: `YYYY-MM-DD-feature-implementation-plan.md`
- Examples: `2026-01-23-tuplespace-implementation-plan.md`

### Design Plans (non-implementation)
- Format: `YYYY-MM-DD-feature-design.md`
- Examples: `2026-01-20-streaming-grammar-push-model-design.md`

### Specifications
- Format: `kebab-case.md`
- Examples: `grammar-system.md`, `vm.md`, `object-system.md`

---

## Document Lifecycle

1. **Draft**: Initial creation, marked as `Status: Draft`
2. **Review**: Circulate for feedback
3. **Approved**: Consensus reached, ready for implementation
4. **In Progress**: Implementation started, update status
5. **Complete**: Implementation done, mark as complete
6. **Deprecated**: Superseded by newer document, add deprecation notice

When marking a document as deprecated:
```markdown
> **DEPRECATED**: This document has been superseded by [New Document](path/to/new.md).
```

---

## Templates

### Empty Design Document Template
```markdown
# [Title]

## Overview
[1-2 sentence description]

## Motivation
[Why this design is needed]

## Design
[Technical description]

### Component 1
[Details]

### Component 2
[Details]

## Examples
```fmpl
[Code examples]
```

## References
- [Related Doc](path)
```

### Empty Implementation Plan Template
```markdown
# [Feature] Implementation Plan

**Status**: Draft
**Date**: YYYY-MM-DD
**Related**: [Design Doc](path)

---

## Overview
[Brief description]

---

## Goals

1. **Goal 1**: [Description]
2. **Goal 2**: [Description]

---

## Non-Goals

- [Out of scope item]

---

## Tasks

### Task 1: [Name]

**File**: `path/to/file`

**Description**:
[What and why]

**Acceptance**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]

---

## Testing

[Testing strategy]

---

## References

- [Related Doc](path)
```
