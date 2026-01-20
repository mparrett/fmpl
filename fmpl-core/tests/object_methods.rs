use fmpl_core::{Value, Vm, eval};

#[test]
fn test_object_methods_use_correct_bodies() {
    let mut vm = Vm::new();
    let _ = eval(
        &mut vm,
        r#"
object web_root {
  entry(): :crossroads
}

object crossroads {
  render_html(): "ok"
}
"#,
    )
    .expect("define objects");

    let entry = eval(&mut vm, "web_root.entry()").expect("entry");
    assert!(matches!(entry, Value::Symbol(sym) if sym == "crossroads"));

    let render = eval(&mut vm, "crossroads.render_html()").expect("render");
    assert!(matches!(render, Value::String(s) if s == "ok"));
}
