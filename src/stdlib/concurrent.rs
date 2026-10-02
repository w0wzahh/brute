use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crate::interpreter::{Interpreter, Value};
use crate::error::{BruteError, Result};

lazy_static::lazy_static! {
    static ref MUTEXES: Mutex<HashMap<i64, Arc<Mutex<Value>>>>                    = Mutex::new(HashMap::new());
    static ref ARCS:    Mutex<HashMap<i64, Arc<Mutex<Value>>>>                    = Mutex::new(HashMap::new());
    static ref THREADS: Mutex<HashMap<i64, JoinHandle<std::result::Result<Value, String>>>> = Mutex::new(HashMap::new());
    static ref CHANNELS: Mutex<HashMap<i64, std::sync::mpsc::Sender<Value>>>      = Mutex::new(HashMap::new());
    static ref RECEIVERS: Mutex<HashMap<i64, std::sync::mpsc::Receiver<Value>>>   = Mutex::new(HashMap::new());
}
static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 { NEXT_ID.fetch_add(1, Ordering::Relaxed) }

fn obj_id(args: &[Value]) -> Result<i64> {
    match args.get(0) {
        Some(Value::Object { fields, .. }) => match fields.get("__id") {
            Some(Value::Int(id)) => Ok(*id),
            _ => Err(BruteError::RuntimeError("object has no handle id".into())),
        },
        Some(Value::Int(id)) => Ok(*id),
        _ => Err(BruteError::RuntimeError("expected a handle object".into())),
    }
}

fn handle_obj(type_name: &str, id: i64, methods: Vec<(&'static str, fn(&mut Interpreter, Vec<Value>) -> Result<Value>)>) -> Value {
    let mut fields = HashMap::new();
    fields.insert("__id".into(), Value::Int(id));
    for (name, func) in methods {
        fields.insert(name.into(), Value::NativeFunction {
            name: format!("{}::{}", type_name, name),
            func,
        });
    }
    Value::Object { type_name: type_name.into(), fields }
}

// ── Mutex methods ────────────────────────────────────────────────────────────

fn with_mutex<R>(id: i64, f: impl FnOnce(&Mutex<Value>) -> R) -> Result<R> {
    let arc = MUTEXES.lock().unwrap().get(&id).cloned()
        .ok_or_else(|| BruteError::RuntimeError(format!("bad mutex handle {}", id)))?;
    Ok(f(&*arc))
}

fn mutex_get(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    with_mutex(id, |m| m.lock().unwrap().clone())
}

fn mutex_set(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let v = a.get(1).cloned().unwrap_or(Value::None);
    with_mutex(id, |m| *m.lock().unwrap() = v)?;
    Ok(Value::None)
}

fn mutex_update(i: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let f = a.get(1).cloned()
        .ok_or_else(|| BruteError::ArgumentError("update() needs a function".into()))?;
    let cur = with_mutex(id, |m| m.lock().unwrap().clone())?;
    let new = i.call_value(f, vec![cur])?;
    with_mutex(id, |m| *m.lock().unwrap() = new.clone())?;
    Ok(new)
}

fn mutex_to_string(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let inner = with_mutex(id, |m| m.lock().unwrap().display())?;
    Ok(Value::String(format!("Mutex({})", inner)))
}

// ── Arc methods ──────────────────────────────────────────────────────────────

fn with_arc<R>(id: i64, f: impl FnOnce(&Arc<Mutex<Value>>) -> R) -> Result<R> {
    let arc = ARCS.lock().unwrap().get(&id).cloned()
        .ok_or_else(|| BruteError::RuntimeError(format!("bad arc handle {}", id)))?;
    Ok(f(&arc))
}

fn arc_get(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    with_arc(id, |m| m.lock().unwrap().clone())
}

fn arc_set(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let v = a.get(1).cloned().unwrap_or(Value::None);
    with_arc(id, |m| *m.lock().unwrap() = v)?;
    Ok(Value::None)
}

fn arc_clone(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let new_id = with_arc(id, |arc| {
        let nid = next_id();
        ARCS.lock().unwrap().insert(nid, Arc::clone(arc));
        nid
    })?;
    Ok(arc_obj(new_id))
}

fn arc_strong_count(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    with_arc(id, |arc| Value::Int(Arc::strong_count(arc) as i64))
}

fn arc_obj(id: i64) -> Value {
    handle_obj("Arc", id, vec![
        ("get", arc_get), ("set", arc_set),
        // `lock()` returns a snapshot of the inner value (e.g. a Mutex object)
        ("lock", arc_get),
        ("clone", arc_clone), ("strong_count", arc_strong_count),
        ("to_string", |_, a| {
            let id = obj_id(&a)?;
            let inner = with_arc(id, |m| m.lock().unwrap().display())?;
            Ok(Value::String(format!("Arc({})", inner)))
        }),
    ])
}

fn mutex_obj(id: i64) -> Value {
    handle_obj("Mutex", id, vec![
        ("get", mutex_get), ("set", mutex_set),
        ("lock", mutex_get),   // lock() returns a snapshot of the inner value
        ("unlock", |_, _| Ok(Value::None)),
        ("update", mutex_update),
        ("to_string", mutex_to_string),
    ])
}

// ── Threads ──────────────────────────────────────────────────────────────────

fn thread_join(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let handle = THREADS.lock().unwrap().remove(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad thread handle {}", id)))?;
    match handle.join() {
        Ok(Ok(v))     => Ok(v),
        Ok(Err(e))    => Err(BruteError::ConcurrencyError(e)),
        Err(_)        => Err(BruteError::ConcurrencyError("thread panicked".into())),
    }
}

fn thread_obj(id: i64) -> Value {
    handle_obj("JoinHandle", id, vec![
        ("join", thread_join),
        ("to_string", |_, a| Ok(Value::String(format!("JoinHandle({})", obj_id(&a)?)))),
    ])
}

// ── Channels ─────────────────────────────────────────────────────────────────

fn channel_send(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let v = a.get(1).cloned().unwrap_or(Value::None);
    let tx = CHANNELS.lock().unwrap().get(&id).cloned()
        .ok_or_else(|| BruteError::RuntimeError(format!("bad channel handle {}", id)))?;
    tx.send(v).map_err(|_| BruteError::ConcurrencyError("channel closed".into()))?;
    Ok(Value::None)
}

fn channel_recv(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let rx = RECEIVERS.lock().unwrap().remove(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad receiver handle {}", id)))?;
    let r = rx.recv().map_err(|_| BruteError::ConcurrencyError("channel closed".into()));
    RECEIVERS.lock().unwrap().insert(id, rx);
    r
}

fn channel_try_recv(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let rx = RECEIVERS.lock().unwrap().remove(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad receiver handle {}", id)))?;
    let r = rx.try_recv().unwrap_or(Value::None);
    RECEIVERS.lock().unwrap().insert(id, rx);
    Ok(r)
}

/// Shared thread-spawn implementation used by both `concurrent` and
/// `async_runtime` modules.
pub fn spawn_fn(a: Vec<Value>) -> Result<Value> {
    match a.get(0).cloned() {
        Some(f @ (Value::Function(_) | Value::NativeFunction { .. })) => {
            let handle = std::thread::spawn(move || {
                let mut interp = Interpreter::new();
                interp.call_value(f, vec![]).map_err(|e| e.to_string())
            });
            let id = next_id();
            THREADS.lock().unwrap().insert(id, handle);
            Ok(thread_obj(id))
        }
        other => Err(BruteError::TypeError(format!(
            "spawn() requires a function, got {}",
            other.map(|v| v.type_name()).unwrap_or("none")))),
    }
}

/// Returns a map of name → Value for the concurrent module.
///
/// `spawn` runs a function on a real OS thread with its own interpreter;
/// `Mutex`/`Arc` provide shared state; `channel` provides mpsc messaging.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("concurrent::", $name).into(),
                func: $body,
            });
        };
    }

    f!("spawn", |_, a| spawn_fn(a));

    f!("sleep", |_, a| {
        let ms = match a.get(0) {
            Some(Value::Int(ms))   => *ms,
            Some(Value::Float(ms)) => *ms as i64,
            _ => 0,
        };
        std::thread::sleep(std::time::Duration::from_millis(ms.max(0) as u64));
        Ok(Value::None)
    });

    // Mutex / Arc constructors as sub-dicts: `Mutex::new(v)`, `Arc::new(v)`
    let mut mutex_dict = HashMap::new();
    mutex_dict.insert("new".into(), Value::NativeFunction {
        name: "Mutex::new".into(),
        func: |_, a| {
            let id = next_id();
            let v = a.get(0).cloned().unwrap_or(Value::None);
            MUTEXES.lock().unwrap().insert(id, Arc::new(Mutex::new(v)));
            Ok(mutex_obj(id))
        },
    });
    m.insert("Mutex".into(), Value::Dict(mutex_dict));

    let mut arc_dict = HashMap::new();
    arc_dict.insert("new".into(), Value::NativeFunction {
        name: "Arc::new".into(),
        func: |_, a| {
            let id = next_id();
            let v = a.get(0).cloned().unwrap_or(Value::None);
            ARCS.lock().unwrap().insert(id, Arc::new(Mutex::new(v)));
            Ok(arc_obj(id))
        },
    });
    m.insert("Arc".into(), Value::Dict(arc_dict));

    // mpsc channel: channel() -> (Sender, Receiver) tuple of objects
    f!("channel", |_, _| {
        let (tx, rx) = std::sync::mpsc::channel::<Value>();
        let tx_id = next_id();
        let rx_id = next_id();
        CHANNELS.lock().unwrap().insert(tx_id, tx);
        RECEIVERS.lock().unwrap().insert(rx_id, rx);
        let sender = handle_obj("Sender", tx_id, vec![
            ("send", channel_send),
            ("to_string", |_, a| Ok(Value::String(format!("Sender({})", obj_id(&a)?)))),
        ]);
        let receiver = handle_obj("Receiver", rx_id, vec![
            ("recv", channel_recv),
            ("try_recv", channel_try_recv),
            ("to_string", |_, a| Ok(Value::String(format!("Receiver({})", obj_id(&a)?)))),
        ]);
        Ok(Value::Tuple(vec![sender, receiver]))
    });

    f!("num_cpus", |_, _| Ok(Value::Int(
        std::thread::available_parallelism().map(|n| n.get() as i64).unwrap_or(1))));

    m
}
