# Backlog Notes

## Protocols and Structural Typing

- Prefer minimal, composable protocols for numeric types, random-access collections, sets, and associative collections.
- Protocols should be definable in user space, with optional runtime-provided implementations for high-performance needs (e.g., swiss tables, ropes).
- Favor structural typing for now; avoid introducing nominal interfaces unless needed for the storylet spike or runtime integration.
