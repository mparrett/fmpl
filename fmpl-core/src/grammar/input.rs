//! PegInput trait and implementations for different input types.
//!
//! This module defines the `PegInput` trait that abstracts over different input sources
//! for PEG parsing. The trait supports:
//! - Text input (character-by-character)
//! - Binary input (byte-by-byte)
//! - Value input (static Vec<Value>)
//! - Stream input (lazy async stream with blocking)
//!
//! The key insight is that position types vary:
//! - Text/Binary/Values use `usize` as position (simple index)
//! - Stream uses `Rc<StreamPosition>` (lazy cons-cell with per-position memo)

use crate::value::Value;
use smol_str::SmolStr;
use std::collections::HashMap;

/// Item returned from input - can be a character, byte, or value.
#[derive(Debug, Clone)]
pub enum InputItem {
    /// A character from text input.
    Char(char),
    /// A byte from binary input.
    Byte(u8),
    /// A value from value/stream input.
    Value(Value),
}

impl InputItem {
    /// Convert to a Value for use in parse results.
    pub fn to_value(&self) -> Value {
        match self {
            InputItem::Char(c) => Value::String(SmolStr::new(c.to_string())),
            InputItem::Byte(b) => Value::Int(*b as i64),
            InputItem::Value(v) => v.clone(),
        }
    }

    /// Get as char if this is a character.
    pub fn as_char(&self) -> Option<char> {
        match self {
            InputItem::Char(c) => Some(*c),
            _ => None,
        }
    }

    /// Get as byte if this is a byte.
    pub fn as_byte(&self) -> Option<u8> {
        match self {
            InputItem::Byte(b) => Some(*b),
            InputItem::Char(c) if c.is_ascii() => Some(*c as u8),
            _ => None,
        }
    }

    /// Get as value reference.
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            InputItem::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Get the byte length when consuming from text (UTF-8).
    pub fn byte_len(&self) -> usize {
        match self {
            InputItem::Char(c) => c.len_utf8(),
            InputItem::Byte(_) => 1,
            InputItem::Value(_) => 1, // Values consume 1 position
        }
    }
}

/// Memoization entry for packrat parsing.
#[derive(Debug, Clone)]
pub enum MemoEntry {
    /// Parsing in progress (for left recursion detection).
    InProgress,
    /// Completed with optional value and end position index.
    Done(Option<Value>, usize),
}

/// Trait for input streams that support PEG parsing.
///
/// This trait abstracts over different input types (text, binary, values, streams).
/// Each implementation provides its own position type and handles memoization.
pub trait PegInput {
    /// The position type for this input.
    /// - Text/Binary/Values: `usize`
    /// - Stream: `Rc<StreamPosition>`
    type Position: Clone;

    /// Get the item at a position, if any (None = end of input).
    fn head(&self, pos: &Self::Position) -> Option<InputItem>;

    /// Advance to the next position.
    fn tail(&self, pos: &Self::Position) -> Self::Position;

    /// Get the numeric index of a position (for result tracking).
    fn index(&self, pos: &Self::Position) -> usize;

    /// Get the position at a given index.
    fn position_at(&self, index: usize) -> Self::Position;

    /// Check if position is at end of input.
    fn is_at_end(&self, pos: &Self::Position) -> bool {
        self.head(pos).is_none()
    }

    /// Get memoization entry for a rule at this position.
    fn get_memo(&self, pos: &Self::Position, rule: &SmolStr) -> Option<MemoEntry>;

    /// Set memoization entry for a rule at this position.
    fn set_memo(&self, pos: &Self::Position, rule: SmolStr, entry: MemoEntry);

    /// Get the starting position.
    fn start(&self) -> Self::Position {
        self.position_at(0)
    }

    // === Text-specific operations (default: unsupported) ===

    /// Get text slice starting at position (for literal matching).
    fn text_from(&self, _pos: &Self::Position) -> Option<&str> {
        None
    }

    /// Check if text at position starts with the given literal.
    fn starts_with(&self, pos: &Self::Position, literal: &str) -> bool {
        self.text_from(pos)
            .is_some_and(|text| text.starts_with(literal))
    }

    // === Binary-specific operations (default: unsupported) ===

    /// Read n bytes starting at position.
    fn bytes_at(&self, _pos: &Self::Position, _n: usize) -> Option<&[u8]> {
        None
    }

    // === Value-specific operations (default: unsupported) ===

    /// Get value at position (for value stream input).
    /// Note: Default implementation returns None. Types that support values
    /// (ValueInput, StreamingInput) override this to return actual references.
    fn value_at(&self, _pos: &Self::Position) -> Option<&Value> {
        None
    }

    /// Check if this input supports text patterns (Char, Literal, CharClass).
    fn supports_text_patterns(&self) -> bool {
        false
    }

    /// Check if this input supports binary patterns (Byte, UInt16BE, etc).
    fn supports_binary_patterns(&self) -> bool {
        false
    }

    /// Check if this input supports value patterns (MatchValue, MatchType, etc).
    fn supports_value_patterns(&self) -> bool {
        false
    }
}

// ============================================================================
// TextInput - Character-by-character text parsing
// ============================================================================

use std::cell::RefCell;

/// Text input for character-by-character parsing.
#[derive(Debug)]
pub struct TextInput {
    text: String,
    memo: RefCell<HashMap<(usize, SmolStr), MemoEntry>>,
}

impl TextInput {
    /// Create a new text input.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            memo: RefCell::new(HashMap::new()),
        }
    }

    /// Get the underlying text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get character at byte position.
    fn char_at_byte(&self, pos: usize) -> Option<char> {
        self.text[pos..].chars().next()
    }
}

impl PegInput for TextInput {
    type Position = usize;

    fn head(&self, pos: &Self::Position) -> Option<InputItem> {
        self.char_at_byte(*pos).map(InputItem::Char)
    }

    fn tail(&self, pos: &Self::Position) -> Self::Position {
        if let Some(c) = self.char_at_byte(*pos) {
            pos + c.len_utf8()
        } else {
            *pos
        }
    }

    fn index(&self, pos: &Self::Position) -> usize {
        *pos
    }

    fn position_at(&self, index: usize) -> Self::Position {
        index
    }

    fn is_at_end(&self, pos: &Self::Position) -> bool {
        *pos >= self.text.len()
    }

    fn get_memo(&self, pos: &Self::Position, rule: &SmolStr) -> Option<MemoEntry> {
        self.memo.borrow().get(&(*pos, rule.clone())).cloned()
    }

    fn set_memo(&self, pos: &Self::Position, rule: SmolStr, entry: MemoEntry) {
        self.memo.borrow_mut().insert((*pos, rule), entry);
    }

    fn text_from(&self, pos: &Self::Position) -> Option<&str> {
        self.text.get(*pos..)
    }

    fn supports_text_patterns(&self) -> bool {
        true
    }

    fn supports_binary_patterns(&self) -> bool {
        // Text can be treated as bytes
        true
    }

    fn bytes_at(&self, pos: &Self::Position, n: usize) -> Option<&[u8]> {
        self.text.as_bytes().get(*pos..*pos + n)
    }
}

// ============================================================================
// BinaryInput - Byte-by-byte binary parsing
// ============================================================================

/// Binary input for byte-by-byte parsing.
#[derive(Debug)]
pub struct BinaryInput {
    bytes: Vec<u8>,
    memo: RefCell<HashMap<(usize, SmolStr), MemoEntry>>,
}

impl BinaryInput {
    /// Create a new binary input.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            memo: RefCell::new(HashMap::new()),
        }
    }

    /// Get the underlying bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PegInput for BinaryInput {
    type Position = usize;

    fn head(&self, pos: &Self::Position) -> Option<InputItem> {
        self.bytes.get(*pos).copied().map(InputItem::Byte)
    }

    fn tail(&self, pos: &Self::Position) -> Self::Position {
        pos + 1
    }

    fn index(&self, pos: &Self::Position) -> usize {
        *pos
    }

    fn position_at(&self, index: usize) -> Self::Position {
        index
    }

    fn is_at_end(&self, pos: &Self::Position) -> bool {
        *pos >= self.bytes.len()
    }

    fn get_memo(&self, pos: &Self::Position, rule: &SmolStr) -> Option<MemoEntry> {
        self.memo.borrow().get(&(*pos, rule.clone())).cloned()
    }

    fn set_memo(&self, pos: &Self::Position, rule: SmolStr, entry: MemoEntry) {
        self.memo.borrow_mut().insert((*pos, rule), entry);
    }

    fn bytes_at(&self, pos: &Self::Position, n: usize) -> Option<&[u8]> {
        self.bytes.get(*pos..*pos + n)
    }

    fn supports_binary_patterns(&self) -> bool {
        true
    }
}

// ============================================================================
// ValueInput - Static Vec<Value> parsing
// ============================================================================

/// Value input for parsing a static list of values.
#[derive(Debug)]
pub struct ValueInput {
    values: Vec<Value>,
    memo: RefCell<HashMap<(usize, SmolStr), MemoEntry>>,
}

impl ValueInput {
    /// Create a new value input.
    pub fn new(values: Vec<Value>) -> Self {
        Self {
            values,
            memo: RefCell::new(HashMap::new()),
        }
    }

    /// Get the underlying values.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Get value at index.
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

impl PegInput for ValueInput {
    type Position = usize;

    fn head(&self, pos: &Self::Position) -> Option<InputItem> {
        self.values.get(*pos).cloned().map(InputItem::Value)
    }

    fn tail(&self, pos: &Self::Position) -> Self::Position {
        pos + 1
    }

    fn index(&self, pos: &Self::Position) -> usize {
        *pos
    }

    fn position_at(&self, index: usize) -> Self::Position {
        index
    }

    fn is_at_end(&self, pos: &Self::Position) -> bool {
        *pos >= self.values.len()
    }

    fn get_memo(&self, pos: &Self::Position, rule: &SmolStr) -> Option<MemoEntry> {
        self.memo.borrow().get(&(*pos, rule.clone())).cloned()
    }

    fn set_memo(&self, pos: &Self::Position, rule: SmolStr, entry: MemoEntry) {
        self.memo.borrow_mut().insert((*pos, rule), entry);
    }

    fn value_at(&self, pos: &Self::Position) -> Option<&Value> {
        self.values.get(*pos)
    }

    fn supports_value_patterns(&self) -> bool {
        true
    }
}

// ============================================================================
// StreamInput wrapper that implements PegInput
// ============================================================================

use super::stream_input::{
    MemoEntry as StreamMemoEntry, StreamInput as RawStreamInput, StreamPosition,
};
use crate::stream::StreamHandle;
use std::rc::Rc;
use std::time::Duration;

/// Streaming input that implements PegInput.
///
/// Wraps the raw StreamInput to provide the PegInput interface.
/// Uses lazy cons-cell positions with per-position memoization.
#[derive(Debug)]
pub struct StreamingInput {
    inner: RawStreamInput,
}

impl StreamingInput {
    /// Create from an async stream handle with default timeout.
    pub fn from_async(handle: StreamHandle) -> Self {
        Self {
            inner: RawStreamInput::from_async(handle),
        }
    }

    /// Create from an async stream handle with custom timeout.
    pub fn from_async_with_timeout(handle: StreamHandle, timeout: Option<Duration>) -> Self {
        Self {
            inner: RawStreamInput::from_async_with_timeout(handle, timeout),
        }
    }

    /// Create from a static list of values (for testing).
    pub fn from_values(values: Vec<Value>) -> Self {
        Self {
            inner: RawStreamInput::from_values(values),
        }
    }
}

impl PegInput for StreamingInput {
    type Position = Rc<StreamPosition>;

    fn head(&self, pos: &Self::Position) -> Option<InputItem> {
        pos.head().cloned().map(InputItem::Value)
    }

    fn tail(&self, pos: &Self::Position) -> Self::Position {
        pos.tail()
    }

    fn index(&self, pos: &Self::Position) -> usize {
        pos.index()
    }

    fn position_at(&self, index: usize) -> Self::Position {
        self.inner.position_at(index)
    }

    fn is_at_end(&self, pos: &Self::Position) -> bool {
        pos.is_at_end()
    }

    fn get_memo(&self, pos: &Self::Position, rule: &SmolStr) -> Option<MemoEntry> {
        pos.get_memo(rule).map(|entry| match entry {
            StreamMemoEntry::InProgress => MemoEntry::InProgress,
            StreamMemoEntry::Done(value, end_index) => MemoEntry::Done(value, end_index),
        })
    }

    fn set_memo(&self, pos: &Self::Position, rule: SmolStr, entry: MemoEntry) {
        let stream_entry = match entry {
            MemoEntry::InProgress => StreamMemoEntry::InProgress,
            MemoEntry::Done(value, end_index) => StreamMemoEntry::Done(value, end_index),
        };
        pos.set_memo(rule, stream_entry);
    }

    fn start(&self) -> Self::Position {
        self.inner.start()
    }

    // Note: value_at returns None for StreamingInput because the value is owned
    // by the StreamPosition, not by self. The runtime uses head() instead which
    // returns an owned InputItem::Value.

    fn supports_value_patterns(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_basic() {
        let input = TextInput::new("hello");
        let pos = input.start();

        assert_eq!(input.index(&pos), 0);
        assert!(!input.is_at_end(&pos));

        let item = input.head(&pos).unwrap();
        assert_eq!(item.as_char(), Some('h'));

        let pos2 = input.tail(&pos);
        assert_eq!(input.index(&pos2), 1);

        let item2 = input.head(&pos2).unwrap();
        assert_eq!(item2.as_char(), Some('e'));
    }

    #[test]
    fn test_text_input_unicode() {
        let input = TextInput::new("héllo");
        let pos = input.start();

        let item = input.head(&pos).unwrap();
        assert_eq!(item.as_char(), Some('h'));

        let pos2 = input.tail(&pos);
        let item2 = input.head(&pos2).unwrap();
        assert_eq!(item2.as_char(), Some('é'));

        // é is 2 bytes in UTF-8
        assert_eq!(input.index(&pos2), 1);
        let pos3 = input.tail(&pos2);
        assert_eq!(input.index(&pos3), 3); // 1 + 2 bytes for é
    }

    #[test]
    fn test_text_input_text_from() {
        let input = TextInput::new("hello world");
        let pos = input.position_at(6);

        assert_eq!(input.text_from(&pos), Some("world"));
        assert!(input.starts_with(&pos, "wor"));
        assert!(!input.starts_with(&pos, "hel"));
    }

    #[test]
    fn test_text_input_memoization() {
        let input = TextInput::new("test");
        let pos = input.start();
        let rule = SmolStr::new("my_rule");

        assert!(input.get_memo(&pos, &rule).is_none());

        input.set_memo(&pos, rule.clone(), MemoEntry::InProgress);
        assert!(matches!(
            input.get_memo(&pos, &rule),
            Some(MemoEntry::InProgress)
        ));

        input.set_memo(&pos, rule.clone(), MemoEntry::Done(Some(Value::Int(42)), 4));
        if let Some(MemoEntry::Done(Some(Value::Int(42)), 4)) = input.get_memo(&pos, &rule) {
            // OK
        } else {
            panic!("expected Done memo entry");
        }
    }

    #[test]
    fn test_binary_input_basic() {
        let input = BinaryInput::new(vec![0x89, 0x50, 0x4E, 0x47]);
        let pos = input.start();

        assert_eq!(input.index(&pos), 0);
        let item = input.head(&pos).unwrap();
        assert_eq!(item.as_byte(), Some(0x89));

        let pos2 = input.tail(&pos);
        assert_eq!(input.index(&pos2), 1);
        let item2 = input.head(&pos2).unwrap();
        assert_eq!(item2.as_byte(), Some(0x50));
    }

    #[test]
    fn test_binary_input_bytes_at() {
        let input = BinaryInput::new(vec![0x01, 0x02, 0x03, 0x04]);
        let pos = input.start();

        let bytes = input.bytes_at(&pos, 2).unwrap();
        assert_eq!(bytes, &[0x01, 0x02]);

        let pos2 = input.position_at(2);
        let bytes2 = input.bytes_at(&pos2, 2).unwrap();
        assert_eq!(bytes2, &[0x03, 0x04]);
    }

    #[test]
    fn test_value_input_basic() {
        let input = ValueInput::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let pos = input.start();

        assert_eq!(input.index(&pos), 0);
        let item = input.head(&pos).unwrap();
        if let InputItem::Value(Value::Int(1)) = item {
            // OK
        } else {
            panic!("expected Value::Int(1)");
        }

        let pos2 = input.tail(&pos);
        assert_eq!(input.index(&pos2), 1);
    }

    #[test]
    fn test_value_input_value_at() {
        let input = ValueInput::new(vec![Value::Int(42), Value::String(SmolStr::new("hello"))]);
        let pos = input.start();

        assert_eq!(input.value_at(&pos), Some(&Value::Int(42)));

        let pos2 = input.tail(&pos);
        assert_eq!(
            input.value_at(&pos2),
            Some(&Value::String(SmolStr::new("hello")))
        );
    }

    #[test]
    fn test_streaming_input_from_values() {
        let input = StreamingInput::from_values(vec![Value::Int(1), Value::Int(2)]);
        let pos = input.start();

        assert_eq!(input.index(&pos), 0);
        let item = input.head(&pos).unwrap();
        if let InputItem::Value(Value::Int(1)) = item {
            // OK
        } else {
            panic!("expected Value::Int(1)");
        }

        let pos2 = input.tail(&pos);
        assert_eq!(input.index(&pos2), 1);

        let pos3 = input.tail(&pos2);
        assert!(input.is_at_end(&pos3));
    }

    #[test]
    fn test_supports_patterns() {
        let text = TextInput::new("test");
        assert!(text.supports_text_patterns());
        assert!(text.supports_binary_patterns());
        assert!(!text.supports_value_patterns());

        let binary = BinaryInput::new(vec![0x00]);
        assert!(!binary.supports_text_patterns());
        assert!(binary.supports_binary_patterns());
        assert!(!binary.supports_value_patterns());

        let values = ValueInput::new(vec![Value::Int(1)]);
        assert!(!values.supports_text_patterns());
        assert!(!values.supports_binary_patterns());
        assert!(values.supports_value_patterns());

        let stream = StreamingInput::from_values(vec![Value::Int(1)]);
        assert!(!stream.supports_text_patterns());
        assert!(!stream.supports_binary_patterns());
        assert!(stream.supports_value_patterns());
    }
}
