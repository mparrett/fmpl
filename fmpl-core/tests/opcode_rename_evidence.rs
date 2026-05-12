//! Evidence tests for SCENARIO-0107 (ITER-0004d.2 T7).
//!
//! ITER-0004d.2 renamed four bytecode `Instruction` variants to reflect
//! post-ITER-0004d.1 semantics:
//! - `MakeTagged` → `MakeListNode`
//! - `ExtractTaggedChild` → `ExtractListChild`
//! - `MatchTagged` → `MatchListNode`
//! - `MatchTaggedWithBindings` → `MatchListNodeWithBindings`
//!
//! `MatchTag` was PRESERVED (it backs `Pattern::Symbol` matching).
//!
//! Wire-format compatibility preserved via `#[serde(rename = "...")]`
//! attributes (Option B from the iteration's binding precondition).
//!
//! This file covers TWO PAR-flagged audit findings:
//!
//! 1. **Dead-code opcode handlers** — `MatchListNode` and
//!    `MatchListNodeWithBindings` have ZERO live emit sites in the current
//!    source tree (their compiler.rs emits were deleted in ITER-0004d.1).
//!    Their VM handlers exist but are unreachable from the sentinel suite.
//!    A typo in either handler would compile + ship undetected because
//!    no live emit path reaches them. These tests construct the bytecode
//!    DIRECTLY (via `Instruction::MatchListNode { ... }` etc.) to confirm
//!    the variants are reachable and the handler dispatch works.
//!
//! 2. **Wire-format Serde round-trip** — `bytecode_persistence.rs` doesn't
//!    exercise any of the four renamed opcodes. A missing or misspelled
//!    `#[serde(rename = ...)]` attribute would silently ship a wire-format
//!    regression. These tests serialize each renamed opcode and assert
//!    the wire-format string is still the OLD name (per the `serde(rename)`
//!    targets).

use fmpl_core::compiler::{CompiledCode, ConstIndex, InstrIndex, Instruction};
use fmpl_core::value::Value;
use fmpl_core::vm::Vm;
use smol_str::SmolStr;
use std::collections::HashMap;

// ============================================================================
// Variant reachability (proves the rename landed; catches stale references)
// ============================================================================

/// SCENARIO-0107 / structural #1: each renamed variant is constructible.
/// If any of these stop compiling, the rename has regressed.
#[test]
fn renamed_variants_are_constructible() {
    // MakeListNode — has a live emit site in builtins/ir.rs:344.
    let _make = Instruction::MakeListNode {
        tag: SmolStr::new("Foo"),
        args: vec![InstrIndex(0)],
    };

    // ExtractListChild — has three live emit sites in compiler.rs and one
    // in builtins/ir.rs.
    let _extract = Instruction::ExtractListChild {
        source: InstrIndex(0),
        index: 0,
    };

    // MatchListNode — DEAD CODE post-ITER-0004d.1 (zero emit sites). This
    // construction is the only path that reaches the variant in the source
    // tree. Without this test, a typo or accidental deletion of the variant
    // ships undetected.
    let _match_node = Instruction::MatchListNode {
        tag_idx: ConstIndex(0),
        patterns: vec![],
    };

    // MatchListNodeWithBindings — DEAD CODE post-ITER-0004d.1. Same rationale.
    let _match_bindings = Instruction::MatchListNodeWithBindings {
        tag_idx: ConstIndex(0),
        bindings: vec![None],
    };
}

/// SCENARIO-0107 / structural #2: MatchTag is preserved (NOT renamed).
/// AC-11 explicitly lists MatchTag among the tagged-bytecode instructions
/// but the iteration chose to preserve it because it backs Pattern::Symbol
/// matching (AC-9 explicitly preserves bare `:foo` symbol literals).
#[test]
fn match_tag_is_preserved() {
    let _match_tag = Instruction::MatchTag {
        value: InstrIndex(0),
        tag: SmolStr::new("Foo"),
        fail_target: InstrIndex(0),
        expected_arity: None,
    };
}

// ============================================================================
// Wire-format Serde round-trip (PAR finding: wire-format coverage gap)
// ============================================================================
//
// `#[serde(rename = "OldName")]` on each renamed variant preserves the wire
// format. The round-trip tests below serialize each variant via serde_json
// and assert the wire string contains the OLD name (the rename target). A
// missing or misspelled `serde(rename)` attribute would surface here as a
// wire-format string with the NEW name — and ITER-0005's persistence layer
// would later fail to deserialize older persisted bytecode.

/// `MakeListNode` wire-format must serialize as `"MakeTagged"`.
#[test]
fn wire_format_makelistnode_serializes_as_maketagged() {
    let instr = Instruction::MakeListNode {
        tag: SmolStr::new("Foo"),
        args: vec![InstrIndex(0)],
    };
    let json = serde_json::to_string(&instr).expect("serialize");
    assert!(
        json.contains("\"MakeTagged\""),
        "MakeListNode must serialize as wire-format `MakeTagged` (Option B \
         #[serde(rename)]). Actual JSON: {json}"
    );
    assert!(
        !json.contains("\"MakeListNode\""),
        "wire format must NOT leak the Rust-side new name `MakeListNode`. \
         Actual JSON: {json}"
    );

    // Round-trip: deserialize back into the renamed variant.
    let restored: Instruction = serde_json::from_str(&json).expect("deserialize");
    assert!(
        matches!(restored, Instruction::MakeListNode { .. }),
        "deserialized variant must be MakeListNode (the renamed Rust name)"
    );
}

/// `ExtractListChild` wire-format must serialize as `"ExtractTaggedChild"`.
#[test]
fn wire_format_extractlistchild_serializes_as_extracttaggedchild() {
    let instr = Instruction::ExtractListChild {
        source: InstrIndex(0),
        index: 0,
    };
    let json = serde_json::to_string(&instr).expect("serialize");
    assert!(
        json.contains("\"ExtractTaggedChild\""),
        "ExtractListChild must serialize as wire-format `ExtractTaggedChild`. \
         Actual JSON: {json}"
    );
    assert!(
        !json.contains("\"ExtractListChild\""),
        "wire format must NOT leak the Rust-side new name. Actual JSON: {json}"
    );
    let restored: Instruction = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(restored, Instruction::ExtractListChild { .. }));
}

/// `MatchListNode` wire-format must serialize as `"MatchTagged"`.
#[test]
fn wire_format_matchlistnode_serializes_as_matchtagged() {
    let instr = Instruction::MatchListNode {
        tag_idx: ConstIndex(0),
        patterns: vec![],
    };
    let json = serde_json::to_string(&instr).expect("serialize");
    assert!(
        json.contains("\"MatchTagged\""),
        "MatchListNode must serialize as wire-format `MatchTagged`. \
         Actual JSON: {json}"
    );
    assert!(
        !json.contains("\"MatchListNode\""),
        "wire format must NOT leak the Rust-side new name. Actual JSON: {json}"
    );
    let restored: Instruction = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(restored, Instruction::MatchListNode { .. }));
}

/// `MatchListNodeWithBindings` wire-format must serialize as
/// `"MatchTaggedWithBindings"`.
#[test]
fn wire_format_matchlistnodewithbindings_serializes_as_matchtaggedwithbindings() {
    let instr = Instruction::MatchListNodeWithBindings {
        tag_idx: ConstIndex(0),
        bindings: vec![None],
    };
    let json = serde_json::to_string(&instr).expect("serialize");
    assert!(
        json.contains("\"MatchTaggedWithBindings\""),
        "MatchListNodeWithBindings must serialize as wire-format \
         `MatchTaggedWithBindings`. Actual JSON: {json}"
    );
    assert!(
        !json.contains("\"MatchListNodeWithBindings\""),
        "wire format must NOT leak the Rust-side new name. Actual JSON: {json}"
    );
    let restored: Instruction = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(
        restored,
        Instruction::MatchListNodeWithBindings { .. }
    ));
}

/// Control: `MatchTag` (preserved unchanged) must serialize as `"MatchTag"`
/// — confirms the variant has no accidental `serde(rename)` attribute.
#[test]
fn wire_format_matchtag_serializes_unchanged() {
    let instr = Instruction::MatchTag {
        value: InstrIndex(0),
        tag: SmolStr::new("Foo"),
        fail_target: InstrIndex(0),
        expected_arity: None,
    };
    let json = serde_json::to_string(&instr).expect("serialize");
    assert!(
        json.contains("\"MatchTag\""),
        "MatchTag (preserved variant) must serialize as `MatchTag`. \
         Actual JSON: {json}"
    );
}

// ============================================================================
// VM-execution coverage (PAR finding: handlers have ZERO live emit sites)
// ============================================================================
//
// The four wire-format tests above prove the `serde(rename)` attributes work
// and the four reachability tests above prove the variants are constructible.
// Neither category routes any value through the VM handler body.
//
// `MatchListNode` and `MatchListNodeWithBindings` have ZERO live emit sites
// in the source tree (their compiler.rs emits were deleted in ITER-0004d.1).
// That means a typo in the handler body — the arity check at vm.rs:2544 or
// vm.rs:2590, the nested dispatch at vm.rs:2608, the binding loop at
// vm.rs:2549-2560 — would compile cleanly and ship undetected because no
// live emit path reaches these handlers.
//
// The tests below CLOSE that gap by constructing bytecode that:
//   1. Builds a list-shaped node via `MakeListNode` (live emit).
//   2. Pushes the node onto the parse_state input stack via `ParsePush`
//      (the only public path to populate parse_state.input from a test).
//   3. Executes `MatchListNode` / `MatchListNodeWithBindings` directly.
//   4. Asserts behavioral correctness (success path, wrong-tag failure,
//      arity-mismatch failure, binding assignment).
//
// Binding assignment is verified via a follow-up `LoadVar(name)` instruction
// — `LoadVar` reads from `frame.locals` first, which is where
// `MatchListNodeWithBindings` writes (vm.rs:2557-2560).

/// Build a minimal CompiledCode with the given instructions and constants.
/// (`stream_coercion.rs::make_code` uses an empty constants table; the tests
/// here need to thread tag and binding names through the constants pool.)
fn make_code_with_constants(instructions: Vec<Instruction>, constants: Vec<Value>) -> CompiledCode {
    CompiledCode {
        instructions,
        nested: vec![],
        source: None,
        constants,
        rule_entry_points: HashMap::new(),
    }
}

/// VM-exec #1: `MatchListNodeWithBindings` succeeds when tag and arity match
/// AND writes bindings into `frame.locals` — verified by `LoadVar` returning
/// the bound value as the program result.
///
/// Bytecode layout:
///   0: LoadInt(7)
///   1: LoadInt(8)
///   2: MakeListNode { tag: "Pair", args: [0, 1] }
///   3: ParsePush { value: 2 }
///   4: MatchListNodeWithBindings { tag_idx: 0("Pair"), bindings: [Some(1), Some(2)] }
///   5: LoadVar("x")
///
/// Constants: [Symbol("Pair"), String("x"), String("y")]
///
/// Expected: run() returns Int(7) — the value bound to "x".
#[test]
fn matchlistnodewithbindings_with_matching_tag_and_arity_binds_correctly() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::LoadInt(7),
            Instruction::LoadInt(8),
            Instruction::MakeListNode {
                tag: SmolStr::new("Pair"),
                args: vec![InstrIndex(0), InstrIndex(1)],
            },
            Instruction::ParsePush {
                value: InstrIndex(2),
            },
            Instruction::MatchListNodeWithBindings {
                tag_idx: ConstIndex(0),
                bindings: vec![Some(ConstIndex(1)), Some(ConstIndex(2))],
            },
            Instruction::LoadVar(SmolStr::new("x")),
        ],
        vec![
            Value::Symbol(SmolStr::new("Pair")),
            Value::String(SmolStr::new("x")),
            Value::String(SmolStr::new("y")),
        ],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Int(7),
        "MatchListNodeWithBindings must bind first child to `x` (got {result:?})"
    );
}

/// VM-exec #1b: companion to #1 — verify the SECOND binding (`y`) is also set.
/// This catches a hypothetical off-by-one bug in the binding loop
/// (vm.rs:2549-2560) that would only ever write to the first slot.
#[test]
fn matchlistnodewithbindings_binds_second_child() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::LoadInt(7),
            Instruction::LoadInt(8),
            Instruction::MakeListNode {
                tag: SmolStr::new("Pair"),
                args: vec![InstrIndex(0), InstrIndex(1)],
            },
            Instruction::ParsePush {
                value: InstrIndex(2),
            },
            Instruction::MatchListNodeWithBindings {
                tag_idx: ConstIndex(0),
                bindings: vec![Some(ConstIndex(1)), Some(ConstIndex(2))],
            },
            Instruction::LoadVar(SmolStr::new("y")),
        ],
        vec![
            Value::Symbol(SmolStr::new("Pair")),
            Value::String(SmolStr::new("x")),
            Value::String(SmolStr::new("y")),
        ],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Int(8),
        "MatchListNodeWithBindings must bind second child to `y` (got {result:?})"
    );
}

/// VM-exec #2: arity mismatch — input has 1 child, bindings expect 2.
/// The arity guard at vm.rs:2544-2547 must fire and set current to Null.
///
/// Bytecode layout:
///   0: LoadInt(7)
///   1: MakeListNode { tag: "Pair", args: [0] }    // arity 1
///   2: ParsePush { value: 1 }
///   3: MatchListNodeWithBindings { tag_idx: "Pair", bindings: [x, y] }  // arity 2
///
/// Expected: run() returns Null (handler sets current to Null at arity mismatch).
#[test]
fn matchlistnodewithbindings_arity_mismatch_fails() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::LoadInt(7),
            Instruction::MakeListNode {
                tag: SmolStr::new("Pair"),
                args: vec![InstrIndex(0)],
            },
            Instruction::ParsePush {
                value: InstrIndex(1),
            },
            Instruction::MatchListNodeWithBindings {
                tag_idx: ConstIndex(0),
                bindings: vec![Some(ConstIndex(1)), Some(ConstIndex(2))],
            },
        ],
        vec![
            Value::Symbol(SmolStr::new("Pair")),
            Value::String(SmolStr::new("x")),
            Value::String(SmolStr::new("y")),
        ],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Null,
        "arity mismatch must set current to Null (got {result:?})"
    );
}

/// VM-exec #3: wrong tag — input is `:Foo(...)`, handler expects `:Bar`.
/// The tag guard at vm.rs:2538-2541 must fire and set current to Null.
#[test]
fn matchlistnodewithbindings_wrong_tag_fails() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::LoadInt(1),
            Instruction::MakeListNode {
                tag: SmolStr::new("Foo"),
                args: vec![InstrIndex(0)],
            },
            Instruction::ParsePush {
                value: InstrIndex(1),
            },
            Instruction::MatchListNodeWithBindings {
                tag_idx: ConstIndex(0), // expects "Bar"
                bindings: vec![Some(ConstIndex(1))],
            },
        ],
        vec![
            Value::Symbol(SmolStr::new("Bar")),
            Value::String(SmolStr::new("v")),
        ],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Null,
        "wrong-tag mismatch must set current to Null (got {result:?})"
    );
}

/// VM-exec #4: `MatchListNode` succeeds when the tag matches and all child
/// patterns succeed. Uses `MatchAny` as wildcard patterns so the nested
/// dispatch at vm.rs:2654-2656 fires for each child.
///
/// Bytecode layout:
///   0: MatchAny                                       // wildcard pattern (placeholder for child 0)
///   1: MatchAny                                       // wildcard pattern (placeholder for child 1)
///   2: LoadInt(1)
///   3: LoadInt(2)
///   4: MakeListNode { tag: "Foo", args: [2, 3] }
///   5: ParsePush { value: 4 }
///   6: MatchListNode { tag_idx: 0("Foo"), patterns: [0, 1] }
///
/// Constants: [Symbol("Foo")]
///
/// Expected: run() returns the matched value (Value::List with tag "Foo").
/// The handler returns `input_val` on success (vm.rs:2670).
///
/// Note: the leading `MatchAny` instructions execute normally first, but with
/// no parse_state input set they fall through the `else` branch at vm.rs:1546
/// and set current to Null — harmless. When the handler later looks them up
/// by index, it treats `MatchAny` as a wildcard (vm.rs:2654-2656).
#[test]
fn matchlistnode_with_matching_tag_succeeds() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::MatchAny,
            Instruction::MatchAny,
            Instruction::LoadInt(1),
            Instruction::LoadInt(2),
            Instruction::MakeListNode {
                tag: SmolStr::new("Foo"),
                args: vec![InstrIndex(2), InstrIndex(3)],
            },
            Instruction::ParsePush {
                value: InstrIndex(4),
            },
            Instruction::MatchListNode {
                tag_idx: ConstIndex(0),
                patterns: vec![InstrIndex(0), InstrIndex(1)],
            },
        ],
        vec![Value::Symbol(SmolStr::new("Foo"))],
    );
    let result = vm.run(&code).expect("VM run failed");
    // Handler sets current to input_val on success.
    assert!(
        result.as_node().is_some(),
        "MatchListNode success must return a list-shaped node (got {result:?})"
    );
    let (tag, children) = result.as_node().unwrap();
    assert_eq!(tag.as_str(), "Foo", "matched node tag must be `Foo`");
    assert_eq!(children.len(), 2, "matched node must have 2 children");
}

/// VM-exec #5: `MatchListNode` arity mismatch — input has 1 child, patterns
/// expects 2. The arity guard at vm.rs:2590-2593 must fire.
#[test]
fn matchlistnode_arity_mismatch_fails() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::MatchAny,
            Instruction::MatchAny,
            Instruction::LoadInt(1),
            Instruction::MakeListNode {
                tag: SmolStr::new("Foo"),
                args: vec![InstrIndex(2)], // arity 1
            },
            Instruction::ParsePush {
                value: InstrIndex(3),
            },
            Instruction::MatchListNode {
                tag_idx: ConstIndex(0),
                patterns: vec![InstrIndex(0), InstrIndex(1)], // arity 2
            },
        ],
        vec![Value::Symbol(SmolStr::new("Foo"))],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Null,
        "MatchListNode arity mismatch must set current to Null (got {result:?})"
    );
}

/// VM-exec #6: `MatchListNode` with wrong tag fails — tag guard at
/// vm.rs:2584-2587 must fire.
#[test]
fn matchlistnode_with_wrong_tag_fails() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::MatchAny,
            Instruction::LoadInt(1),
            Instruction::MakeListNode {
                tag: SmolStr::new("Foo"),
                args: vec![InstrIndex(1)],
            },
            Instruction::ParsePush {
                value: InstrIndex(2),
            },
            Instruction::MatchListNode {
                tag_idx: ConstIndex(0), // expects "Bar"
                patterns: vec![InstrIndex(0)],
            },
        ],
        vec![Value::Symbol(SmolStr::new("Bar"))],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Null,
        "MatchListNode wrong-tag mismatch must set current to Null (got {result:?})"
    );
}

/// VM-exec #7: `MatchListNode` with a nested `MatchListNodeWithBindings`
/// pattern — exercises the inlined nested dispatch branch at vm.rs:2608-2645
/// (the inlined re-implementation of the bindings handler). A bug in the
/// inlined version (different from the standalone handler) would only surface
/// through this path.
///
/// Bytecode layout — input: `[Foo, [Bar, 5]]` (one child that is `:Bar(5)`).
///   0: MatchListNodeWithBindings { tag_idx: 1("Bar"), bindings: [Some(2)("z")] }
///      (placeholder — executes as a regular instruction first, but with no
///       parse_state input it falls through to set current to Null; harmless.)
///   1: LoadInt(5)
///   2: MakeListNode { tag: "Bar", args: [1] }
///   3: MakeListNode { tag: "Foo", args: [2] }
///   4: ParsePush { value: 3 }
///   5: MatchListNode { tag_idx: 0("Foo"), patterns: [0] }
///   6: LoadVar("z")
///
/// Constants: [Symbol("Foo"), Symbol("Bar"), String("z")]
///
/// Expected: run() returns Int(5) — z bound by the nested MatchListNodeWithBindings.
#[test]
fn matchlistnode_with_nested_bindings_pattern() {
    let mut vm = Vm::new();
    let code = make_code_with_constants(
        vec![
            Instruction::MatchListNodeWithBindings {
                tag_idx: ConstIndex(1),
                bindings: vec![Some(ConstIndex(2))],
            },
            Instruction::LoadInt(5),
            Instruction::MakeListNode {
                tag: SmolStr::new("Bar"),
                args: vec![InstrIndex(1)],
            },
            Instruction::MakeListNode {
                tag: SmolStr::new("Foo"),
                args: vec![InstrIndex(2)],
            },
            Instruction::ParsePush {
                value: InstrIndex(3),
            },
            Instruction::MatchListNode {
                tag_idx: ConstIndex(0),
                patterns: vec![InstrIndex(0)],
            },
            Instruction::LoadVar(SmolStr::new("z")),
        ],
        vec![
            Value::Symbol(SmolStr::new("Foo")),
            Value::Symbol(SmolStr::new("Bar")),
            Value::String(SmolStr::new("z")),
        ],
    );
    let result = vm.run(&code).expect("VM run failed");
    assert_eq!(
        result,
        Value::Int(5),
        "nested MatchListNodeWithBindings pattern must bind `z` to 5 (got {result:?})"
    );
}
