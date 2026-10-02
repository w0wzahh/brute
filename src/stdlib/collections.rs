use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use crate::interpreter::{Interpreter, Value};
use crate::error::{BruteError, Result};

lazy_static::lazy_static! {
    // Shared backing stores — lets `set.add(x)`/`pq.dequeue()` mutate in place.
    static ref SETS: Mutex<HashMap<i64, Vec<Value>>>          = Mutex::new(HashMap::new());
    static ref PQS:  Mutex<HashMap<i64, Vec<(i64, Value)>>>   = Mutex::new(HashMap::new());
}
static NEXT_ID: AtomicI64 = AtomicI64::new(1);
fn next_id() -> i64 { NEXT_ID.fetch_add(1, Ordering::Relaxed) }

fn obj_id(args: &[Value]) -> Result<i64> {
    match args.get(0) {
        Some(Value::Object { fields, .. }) => match fields.get("__id") {
            Some(Value::Int(id)) => Ok(*id),
            _ => Err(BruteError::RuntimeError("object has no handle id".into())),
        },
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

fn set_obj(items: Vec<Value>) -> Value {
    let id = next_id();
    let mut uniq: Vec<Value> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for v in items {
        if seen.insert(v.repr()) { uniq.push(v); }
    }
    SETS.lock().unwrap().insert(id, uniq);
    handle_obj("Set", id, vec![
        ("add", set_add), ("remove", set_remove), ("contains", set_contains),
        ("to_list", set_to_list), ("to_string", set_to_list),
        ("size", set_size), ("len", set_size), ("is_empty", set_is_empty),
        ("intersection", set_intersection), ("union", set_union), ("difference", set_difference),
    ])
}

fn set_items(args: &[Value]) -> Result<Vec<Value>> {
    let id = obj_id(args)?;
    SETS.lock().unwrap().get(&id).cloned()
        .ok_or_else(|| BruteError::RuntimeError(format!("bad set handle {}", id)))
}

fn set_add(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    if let Some(v) = a.get(1) {
        let mut sets = SETS.lock().unwrap();
        let items = sets.get_mut(&id)
            .ok_or_else(|| BruteError::RuntimeError(format!("bad set handle {}", id)))?;
        if !items.iter().any(|x| x.repr() == v.repr()) { items.push(v.clone()); }
    }
    Ok(a.get(0).cloned().unwrap_or(Value::None))
}

fn set_remove(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let v = a.get(1).cloned().unwrap_or(Value::None);
    let mut sets = SETS.lock().unwrap();
    let items = sets.get_mut(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad set handle {}", id)))?;
    match items.iter().position(|x| x.repr() == v.repr()) {
        Some(i) => Ok(items.remove(i)),
        None    => Ok(Value::None),
    }
}

fn set_contains(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let v = a.get(1).cloned().unwrap_or(Value::None);
    let items = set_items(&a)?;
    Ok(Value::Bool(items.iter().any(|x| x.repr() == v.repr())))
}

fn set_to_list(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    Ok(Value::List(set_items(&a)?))
}

fn set_size(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    Ok(Value::Int(set_items(&a)?.len() as i64))
}

fn set_is_empty(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    Ok(Value::Bool(set_items(&a)?.is_empty()))
}

fn other_set_items(a: &[Value]) -> Result<Vec<Value>> {
    let other = a.get(1).cloned().unwrap_or(Value::None);
    set_items(&[other])
}

fn set_combine(a: Vec<Value>, keep: impl Fn(&Value, &[Value]) -> bool) -> Result<Value> {
    let mine = set_items(&a)?;
    let theirs = other_set_items(&a)?;
    let out: Vec<Value> = mine.into_iter().filter(|v| keep(v, &theirs)).collect();
    Ok(set_obj(out))
}

fn set_intersection(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    set_combine(a, |v, t| t.iter().any(|x| x.repr() == v.repr()))
}
fn set_difference(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    set_combine(a, |v, t| !t.iter().any(|x| x.repr() == v.repr()))
}
fn set_union(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let mut out = set_items(&a)?;
    for v in other_set_items(&a)? {
        if !out.iter().any(|x| x.repr() == v.repr()) { out.push(v); }
    }
    Ok(set_obj(out))
}

fn pq_obj() -> Value {
    let id = next_id();
    PQS.lock().unwrap().insert(id, vec![]);
    handle_obj("PriorityQueue", id, vec![
        ("enqueue", pq_enqueue), ("dequeue", pq_dequeue), ("peek", pq_peek),
        ("size", pq_size), ("len", pq_size), ("is_empty", pq_is_empty),
    ])
}

fn pq_entries(args: &[Value]) -> Result<Vec<(i64, Value)>> {
    let id = obj_id(args)?;
    PQS.lock().unwrap().get(&id).cloned()
        .ok_or_else(|| BruteError::RuntimeError(format!("bad pq handle {}", id)))
}

fn pq_enqueue(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let item = a.get(1).cloned().unwrap_or(Value::None);
    let prio = match a.get(2) { Some(Value::Int(p)) => *p, _ => 0 };
    PQS.lock().unwrap().get_mut(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad pq handle {}", id)))?
        .push((prio, item));
    Ok(a.get(0).cloned().unwrap_or(Value::None))
}

fn pq_peek(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let entries = pq_entries(&a)?;
    Ok(entries.iter().max_by_key(|(p, _)| *p).map(|(_, v)| v.clone()).unwrap_or(Value::None))
}

fn pq_dequeue(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    let id = obj_id(&a)?;
    let mut pqs = PQS.lock().unwrap();
    let entries = pqs.get_mut(&id)
        .ok_or_else(|| BruteError::RuntimeError(format!("bad pq handle {}", id)))?;
    match entries.iter().enumerate().max_by_key(|(_, (p, _))| *p).map(|(i, _)| i) {
        Some(i) => Ok(entries.remove(i).1),
        None    => Ok(Value::None),
    }
}

fn pq_size(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    Ok(Value::Int(pq_entries(&a)?.len() as i64))
}
fn pq_is_empty(_: &mut Interpreter, a: Vec<Value>) -> Result<Value> {
    Ok(Value::Bool(pq_entries(&a)?.is_empty()))
}

pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    m.insert("new_list".into(), Value::NativeFunction {
        name: "collections::new_list".into(),
        func: |_interp, _args| Ok(Value::List(vec![])),
    });

    m.insert("new_dict".into(), Value::NativeFunction {
        name: "collections::new_dict".into(),
        func: |_interp, _args| Ok(Value::Dict(HashMap::new())),
    });

    m.insert("sorted".into(), Value::NativeFunction {
        name: "collections::sorted".into(),
        func: |interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(mut v) => {
                    v.sort_by(|a, b| interp.compare(a, b)
                        .map(|o| o.cmp(&0))
                        .unwrap_or(std::cmp::Ordering::Equal));
                    Ok(Value::List(v))
                }
                other => Err(BruteError::TypeError(format!("sorted() requires a list, got {}", other.type_name()))),
            }
        },
    });

    m.insert("reversed".into(), Value::NativeFunction {
        name: "collections::reversed".into(),
        func: |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(mut v) => { v.reverse(); Ok(Value::List(v)) }
                other => Err(BruteError::TypeError(format!("reversed() requires a list, got {}", other.type_name()))),
            }
        },
    });

    m.insert("flatten".into(), Value::NativeFunction {
        name: "collections::flatten".into(),
        func: |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(v) => {
                    let mut out = Vec::new();
                    for item in v {
                        match item {
                            Value::List(inner) => out.extend(inner),
                            other              => out.push(other),
                        }
                    }
                    Ok(Value::List(out))
                }
                other => Err(BruteError::TypeError(format!("flatten() requires a list, got {}", other.type_name()))),
            }
        },
    });

    m.insert("unique".into(), Value::NativeFunction {
        name: "collections::unique".into(),
        func: |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(v) => {
                    let mut seen  = Vec::new();
                    let mut out   = Vec::new();
                    for item in v {
                        let repr = item.repr();
                        if !seen.contains(&repr) {
                            seen.push(repr);
                            out.push(item);
                        }
                    }
                    Ok(Value::List(out))
                }
                other => Err(BruteError::TypeError(format!("unique() requires a list, got {}", other.type_name()))),
            }
        },
    });

    // Type constructors — `collections.Vector([..])`, `collections.HashMap()`
    m.insert("Vector".into(), Value::NativeFunction {
        name: "collections::Vector".into(),
        func: |_interp, args| {
            match args.into_iter().next() {
                Some(Value::List(v)) => Ok(Value::List(v)),
                Some(other)          => Ok(Value::List(vec![other])),
                None                 => Ok(Value::List(vec![])),
            }
        },
    });

    m.insert("HashMap".into(), Value::NativeFunction {
        name: "collections::HashMap".into(),
        func: |_interp, _args| Ok(Value::Dict(HashMap::new())),
    });

    m.insert("Stack".into(), Value::NativeFunction {
        name: "collections::Stack".into(),
        func: |_interp, args| {
            match args.into_iter().next() {
                Some(Value::List(v)) => Ok(Value::List(v)),
                _                  => Ok(Value::List(vec![])),
            }
        },
    });

    m.insert("Queue".into(), Value::NativeFunction {
        name: "collections::Queue".into(),
        func: |_interp, args| {
            match args.into_iter().next() {
                Some(Value::List(v)) => Ok(Value::List(v)),
                _                  => Ok(Value::List(vec![])),
            }
        },
    });

    m.insert("Set".into(), Value::NativeFunction {
        name: "collections::Set".into(),
        func: |_interp, args| {
            let items = match args.into_iter().next() {
                Some(Value::List(v)) => v,
                Some(other)          => vec![other],
                None                 => vec![],
            };
            Ok(set_obj(items))
        },
    });

    m.insert("PriorityQueue".into(), Value::NativeFunction {
        name: "collections::PriorityQueue".into(),
        func: |_interp, _args| Ok(pq_obj()),
    });

    m
}
