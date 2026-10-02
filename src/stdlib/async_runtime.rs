use std::collections::HashMap;
use crate::interpreter::{Interpreter, Value};
use crate::error::{BruteError, Result};

fn block_on_impl(i: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    match a.get(0).cloned() {
        // A callable — invoke it and return its result
        Some(f @ (Value::Function(_) | Value::NativeFunction { .. })) => i.call_value(f, vec![]),
        // An already-computed value (async fns evaluate eagerly in the
        // tree-walking interpreter) — pass it through
        Some(v) => Ok(v),
        None => Ok(Value::None),
    }
}

fn runtime_obj() -> Value {
    let mut fields = HashMap::new();
    fields.insert("block_on".into(), Value::NativeFunction {
        name: "AsyncRuntime::block_on".into(),
        func: |i, a| {
            // args[0] is the runtime object itself (method dispatch);
            // the awaited value is args[1]
            let mut rest = a;
            if !rest.is_empty() { rest.remove(0); }
            block_on_impl(i, rest)
        },
    });
    fields.insert("to_string".into(), Value::NativeFunction {
        name: "AsyncRuntime::to_string".into(),
        func: |_, _| Ok(Value::String("AsyncRuntime".into())),
    });
    Value::Object { type_name: "AsyncRuntime".into(), fields }
}

/// Returns a map of name → Value for the async_runtime module.
///
/// Brute's `async`/`await` evaluates synchronously in the tree-walking
/// interpreter — `await e` yields `e` directly. `AsyncRuntime::block_on`
/// and `spawn`/`join` provide the familiar runtime API over that model.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    let mut rt = HashMap::new();
    rt.insert("new".into(), Value::NativeFunction {
        name: "AsyncRuntime::new".into(),
        func: |_, _| Ok(runtime_obj()),
    });
    m.insert("AsyncRuntime".into(), Value::Dict(rt));

    m.insert("block_on".into(), Value::NativeFunction {
        name: "async_runtime::block_on".into(),
        func: block_on_impl,
    });
    m.insert("sleep".into(), Value::NativeFunction {
        name: "async_runtime::sleep".into(),
        func: |_, a| {
            let ms = match a.get(0) {
                Some(Value::Int(ms))   => *ms,
                Some(Value::Float(ms)) => *ms as i64,
                _ => 0,
            };
            std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
            Ok(Value::None)
        },
    });
    // Spawn a task on a real OS thread (shares concurrent's thread registry
    // via a fresh interpreter — same semantics as concurrent::spawn)
    m.insert("spawn".into(), Value::NativeFunction {
        name: "async_runtime::spawn".into(),
        func: |_, a| crate::stdlib::concurrent::spawn_fn(a),
    });

    m
}
