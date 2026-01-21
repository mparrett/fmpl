study spec/README.md, docs/plans/2026-01-19-unified-grammars-and-agents-design.md and docs/plans/12-layer-human-ai-architecture.md

Work on the next thing that will move the needle towards having a working ratatui agentic app.

REMEMBER:
- Use TDD principles: write tests for new functionality before implementing it.
- Document your code and decisions clearly for future reference. 
  - If the specs or documentation are lacking, you MUST improve them to make the next iteration easier. 
  - Context is king. Keep our working set as minimal as possible while still providing enough context to understand the changes.
- To exercise the grammars, writing scannerless parsers for things like JSON in FMPL is a great way to validate them.
- Confirm features are missing before deciding they're out of scope.
- Focus on incremental progress; small, testable changes are better than large, untested ones.
- With FMPL, prioritize correctness and clarity over cleverness.
- Try to think of lightweight reusable grannars to make integrations easier.
- Keep the end goal in mind: a functional ratatui agentic application.
- Fix all warnings and errors you encounter, including clippy warnings.
- Update the specs and documentation as you make changes
- You have cart blance to make decisions on architecture and implementation details, but justify your choices based on the goals of the project. You must use the FMPL vm, however, and you must use valid bytecode within it.
- TUI work must be done using ratatui.
- Use t-shirt sizes for complexity, rather than time estimates.
