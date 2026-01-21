# 12 Layer agent architecture for Human-AI Collaboration

[HumanLayer](https://github.com/humanlayer/12-factor-agents) has some guidelines on how to build more predictable agentic systems.

We want to be able to leverage our chains of grammars and reactive steams to make it easier to build complex Human-AI collaborative systems. To do this, we can define a 12-layer architecture that breaks down the components needed for such systems.

This should act as a combination of a moldable development environment, combining:
1. debugging
2. repl and dynamic code execution
3. prompt and context editing an management
4. conversational agent management per interface

## Layer 1: Input Layer
I'd like to borrow from the humanlayer design patterns here.

This requires:
1. A research view of the problem space, which is assembled into a context for the next layer
2. A planning view, which given the broader context of the research view, works collaboratively with the user to find the right scope for implementation.
3. Given the planning view, an execution view that breaks down the plan into actionable steps, using coding agents, review agents, and verification/judgment from the user or an LLM proxy.

## Layer 2: Contextual Layer

The human layer systems require the ability to backtrack and edit the historical context of a prior panels as a form of active, continuous compaction. Some combination of explicit user input and automated detection of the LLM agents when they are going off track ("You're absolulely right") should be a signal to backtrack, revise with the new user feedback, and re-issue the prompt to the LLM agents.

Revision should also possible to elide tool calls and MCP calls, so only the actual relevant information and context is retained. This might require something like a VCS of the conversation history, or a clever data structure that allows for branching and merging of conversation threads.

## Layer 3: Agent description/datayflow

We shoudl be able to use the FMPL language to cleaning describe tasks involving data manipulation, transforming source input into proper formats, triggering semantic actions, etc. Provide links to the other documents within this archive.

## Layer 4: Tooling Layer

Tools should be written with a mixture of internal scripting (FMPL) as well as potentially having external calls to other hosts/services. The tools should be able to be composed together in a chain, with clear input/output specifications. The codebase has a link to curl.rs, so many protocols are lightly covered out of the box.

## The UI itself

We should have a set of UI components that allow for easy construction of these layers. This includes:
1. A panel system that allows for multiple views (research, planning, execution) to be visible and interactive.
2. A context editor that allows users to backtrack and revise prior context.
3. A tool management interface that allows users to add, remove, and configure tools - this probably could use the internal VM, but also allow for external tool integration using MCP/ACP, etc.

Since we're in rust, let's use ratatui for a text based version first. We can always build a desktop/web version later, leveraging the fmpl-web crate.
