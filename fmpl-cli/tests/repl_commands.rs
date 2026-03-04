//! Tests for REPL commands
//!
//! Tests the REPL command functionality including object listing.

use fmpl_core::{Vm, eval};

/// Helper to verify that named objects are correctly tracked
#[test]
fn test_named_objects_tracking() {
    let mut vm = Vm::new();

    // Initially, no named objects
    let count = vm.objects.lock().unwrap().named_objects().count();
    assert_eq!(count, 0, "Expected no named objects initially");

    // The @name registration in object definitions creates named objects
    // Define an object with a name - this should register it
    let _ = eval(
        &mut vm,
        r#"
object test_obj {
  get_value(): 42
}
"#,
    )
    .expect("define object");

    // After defining an object, it should be registered by name
    let db = vm.objects.lock().unwrap();
    let named: Vec<_> = db
        .named_objects()
        .map(|(name, id)| (name.to_owned(), *id))
        .collect();
    drop(db);

    assert!(
        !named.is_empty(),
        "Expected at least 1 named object after object definition"
    );

    // One of them should be "test_obj" (the object we just defined)
    let names: Vec<_> = named.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        names.contains(&"test_obj"),
        "Expected 'test_obj' to be registered. Found: {:?}",
        names
    );
}

/// Test that multiple named objects can be tracked
#[test]
fn test_multiple_named_objects() {
    let vm = Vm::new();

    // Create multiple objects and register them
    let obj1_id = vm.objects.lock().unwrap().create(None);
    let obj2_id = vm.objects.lock().unwrap().create(None);
    let obj3_id = vm.objects.lock().unwrap().create(None);

    vm.objects
        .lock()
        .unwrap()
        .register_name("first".into(), obj1_id);
    vm.objects
        .lock()
        .unwrap()
        .register_name("second".into(), obj2_id);
    vm.objects
        .lock()
        .unwrap()
        .register_name("third".into(), obj3_id);

    // Keep lock guard alive while collecting data
    let db = vm.objects.lock().unwrap();
    let named: Vec<_> = db
        .named_objects()
        .map(|(name, id)| (name.to_owned(), *id))
        .collect();
    drop(db);

    assert_eq!(named.len(), 3);

    // Names should be present (order may vary due to HashMap)
    let names: Vec<_> = named.iter().map(|(name, _)| name.as_str()).collect();
    assert!(names.contains(&"first"));
    assert!(names.contains(&"second"));
    assert!(names.contains(&"third"));
}

/// Test that named_objects handles empty case gracefully
#[test]
fn test_empty_named_objects() {
    let vm = Vm::new();
    let count = vm.objects.lock().unwrap().named_objects().count();
    assert_eq!(count, 0, "Expected no named objects in fresh VM");
}
