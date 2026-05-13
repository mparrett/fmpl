# Discord bot spike — pre-iteration prep

**Date:** 2026-05-13
**Status:** prep notes. The Discord bot iteration is scheduled to start at the end of ITER-0005 (or at hard-stop end-of-day 2026-05-13 if persistence work runs over). This doc captures decisions made during pre-iteration discussion so the picking-up agent doesn't re-derive them.
**Author:** Norman + Claude (joint discussion 2026-05-13)

## Decision summary

| Decision | Value | Status |
|---|---|---|
| Pivot from fmpl-web spike to Discord bot | Yes | Confirmed |
| Number of Discord servers | 2 | Confirmed |
| Users per server | ~10-20 each (~30-40 total) | Confirmed |
| Crate placement | new `fmpl-discord/` crate (not a bin inside `fmpl-web`) | Confirmed |
| Token loading | `.env` OR local environment (`DISCORD_TOKEN` env var) | Confirmed |
| Default VM scope | "one shared VM across both servers" — Option 1 of the discussion | Confirmed, with capabilities qualifier (see open question) |
| Stop-rule for ITER-0005a.2 (preceding work) | B — stop at next clean task boundary after projection-vs-cutoff signal | Confirmed |
| Hard cutoff for persistence work | 2026-05-13T22:00Z (~18:00 EDT) unless renegotiated | Confirmed |

## Why a Discord bot beats the fmpl-web spike for the Friday demo

The user's stated real want is **"multi-user collaboration with concurrency."** The fmpl-web spike requires building the multi-user piece (WebSocket fan-out, two-tab session management, etc.); Discord channels are inherently multi-user. The bot trades visual polish (no fancy slot-source UI in a chat transcript) for substrate-cost (no per-user-session plumbing to build).

The fmpl-web work is not wasted by this pivot — it stays on the branch, still has value, still progresses toward eventual collaborative authoring. Friday delivery and long-term direction are separable.

## Anticipated phase plan

These are anticipated phases, not finalized scope. A scope card will be PAR-reviewed at iteration entry per the project's iterative-development discipline.

### Phase 1 — Bot core
- `cargo new fmpl-discord --bin`
- `serenity = "0.12"` with `client`, `gateway`, `model`, `rustls_backend` features
- `EventHandler::message`: parse `!fmpl ` prefix; eval the rest against a shared `Arc<Mutex<Vm>>`; reply in the same channel
- Token loading: prefer `.env` via `dotenvy`; fall back to plain `std::env::var("DISCORD_TOKEN")`
- Verification: two users in a channel exchange `!fmpl` commands; state persists between commands

### Phase 2 — Multi-line + Discord-shape polish
- Multi-line FMPL via fenced code blocks (```\`\`\`fmpl ... \`\`\````)
- Output formatting: Discord code blocks, truncation at 2000-char limit
- `!fmpl objects`: list every object via `ObjectDb` + per-object one-line summary
- `!fmpl src <name>`: show source via `object_source_repr`
- `!fmpl whoami`: print Discord username + guild context (makes cross-server angle visible)
- Logging: `(timestamp, guild_id, channel_id, user_id, username, command)` per command — local file, not user-visible

### Phase 3 — Concurrency demo prep
- `!fmpl history`: last 10 commands across all servers, showing `username@guild: command`. **This is the concurrency demo:** shared state across two servers visible in one query.
- Bot announces itself in each channel on startup
- `!fmpl help`: short reference card
- A prepared demo sequence for the Friday accountability check: two laptops/tabs open to the two servers; user A defines a thing in server A → server B queries it → server B mutates → server A re-queries → `!fmpl history` shows interleaved activity

### Phase 4 — Optional polish (only if time)
- Persistence across bot restarts (reuse `ImageStore` from fmpl-web, OR save the VM state to disk on every command)
- Slash commands (`/fmpl ...`) vs. text-prefix commands
- Multi-user permissions / role gating (e.g., who can run `!fmpl reset`)

## What's OUT of scope for the Friday demo

- Voice channels, threads, DMs, reactions — text channels only.
- Per-user FMPL vats. The single shared VM IS the demo (it's collaborative-by-construction).
- A Discord bot that writes FMPL programs itself — separate aspiration, not Friday.
- Web UI integration — the fmpl-web spike stays paused.

## Open question carried over: the capabilities model

User raised during pre-iteration discussion: **"perhaps we should have the capabilities model in play here?"** — referring to Option 1 (one shared VM across both servers) and asking whether capability-scoping should constrain what cross-server users can do.

The discussion was interrupted before a concrete capabilities design was agreed. This is the load-bearing design question for the bot iteration. Two framings worth investigating at iteration entry:

**Framing A — Per-principal capabilities.** Each `(guild_id, user_id)` is a principal. Some capabilities (read VM state, define new objects) granted broadly; others (`!fmpl reset`, mutating shared objects, deleting objects) gated by capability presence. Bot owners hand out capabilities explicitly.

**Framing B — Per-object capabilities.** Objects in the VM have ACLs ("only user X can mutate `foo`"). Read is universal; write is per-object. Aligns with object-capability discipline (`feedback_*` memory on capability discipline doesn't yet exist; would be created if this framing is adopted).

**Framing C — Per-channel namespacing.** Each channel has its own object-space scope; cross-channel reference requires explicit capability tokens. Closer to ocap (object capabilities) — the channel IS the security boundary.

**Framing D — None for the demo.** Single shared VM, no capability model, accept that any user can `!fmpl reset` and the whole world goes away. Document the limitation. Add capability model in a follow-up iteration once the bare bot is shipping.

The user's framing in the conversation leaned toward "capabilities model is right but I haven't pinned which form" — so the answer **probably isn't D long-term**, but D is the smallest-risk Friday-demo choice. Bot iteration's PAR scope review should explicitly probe this.

## Connection to other in-flight design directions

- **`docs/superpowers/specs/2026-05-13-prolog-shaped-ffi.md`** §Phase 2-4 sketches FFI-as-tuple-space with method-dispatch-via-roles. A `make_relation(:Capability, 2)` + `assert Capability(user, "reset")` natively expresses Framing A above. Worth keeping the two designs aligned — the bot's capability model is one concrete consumer of the eventual tuple-space FFI.
- **`feedback_ship_infrastructure_with_first_consumer.md`** discipline applies: if the bot iteration becomes the first real consumer of capability infrastructure, then the capability model ships with the bot (not before).
- **`feedback_split_iterations_on_reader_writer_asymmetry.md`** discipline applies: bot iteration likely splits into "Phase 1+2 core+polish" + "Phase 3 history/observability" + "capability model" rather than one umbrella iteration.

## Concrete prerequisites (only the user can do these)

Before the bot iteration can start implementation:

1. **Create a Discord application + bot** at https://discord.com/developers/applications. Save the token securely.
2. **Enable the Message Content intent** on the bot (privileged gateway intent, requires toggle in the developer portal).
3. **Invite the bot to both target servers** via the OAuth2 URL generator (scopes: `bot`; permissions: `Send Messages`, `Read Message History`).
4. **Provide the token** to the running binary via `DISCORD_TOKEN` env var or `.env` file in the workspace root. The bot will read either.

Without prerequisites 1-4, the bot iteration cannot start its T0 (Phase 1). Doing them is ~5 minutes of clicking.

## Stop-rule cross-reference

Per the iteration discipline in flight 2026-05-13:

- **Hard stop on ITER-0005a.2** is 2026-05-13T22:00Z (~18:00 EDT) unless renegotiated.
- **At that stop**, whatever state ITER-0005a.2 is in becomes the boundary:
  - If complete (T0-T6 + audit clean): ship as a normal close, then start Discord bot.
  - If mid-task: roll back any uncommitted partial-task work to the previous clean task boundary; commit T0 (or whichever tasks completed cleanly); stop. Resume ITER-0005a.2 next week.
- **Discord bot iteration starts when:** the persistence work boundary is committed AND user has completed the Discord-side prerequisites above.

## Files referenced for the bot iteration's eventual scope card

- `fmpl-web/src/main.rs` — existing REPL eval pattern (`eval_code` async handler, `Arc<Mutex<Vm>>` state)
- `fmpl-web/src/main.rs:247` — WebSocket fan-out pattern (`approval_ws_handler`) — pattern reusable for bot-side observability if needed
- `fmpl-core/src/vm.rs:3415` — existing FFI surface (`call_builtin`)
- `fmpl-core/src/object.rs` — `ObjectDb` for `!fmpl objects` listing
- `fmpl-core/src/repr.rs:792` — `object_source_repr` for `!fmpl src <name>`
- `docs/superpowers/specs/2026-05-13-prolog-shaped-ffi.md` — long-term FFI direction; the bot iteration's capability-model choice should not foreclose this
