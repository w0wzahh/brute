use std::collections::HashMap;
use std::sync::Arc;
use std::fmt;
use std::io::{self, BufRead, Write};
use crate::ast::*;
use crate::error::{BruteError, Result};
use crate::traits::TraitRegistry;
use crate::module_system::ModuleRegistry;

// ─────────────────────────────────────────────────────────────────────────────
// Value
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Tuple(Vec<Value>),
    Range { start: i64, end: i64, inclusive: bool },
    Object { type_name: String, fields: HashMap<String, Value> },
    Enum   { type_name: String, variant: String, fields: Vec<Value> },
    Function(Function),
    NativeFunction { name: String, func: fn(&mut Interpreter, Vec<Value>) -> Result<Value> },
    None,
    Return(Box<Value>),   // internal control-flow token
    Break(Box<Value>),    // internal control-flow token
    Continue,             // internal control-flow token
}

#[derive(Clone)]
pub struct Function {
    pub name:    String,
    pub params:  Vec<Param>,
    pub body:    Vec<Stmt>,
    /// Captured environment, shared cheaply via `Arc` — deep-cloning the env
    /// per function grows exponentially since closures contain closures.
    pub closure: Arc<Env>,
    pub is_async: bool,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_)             => "int",
            Value::Float(_)           => "float",
            Value::Bool(_)            => "bool",
            Value::String(_)          => "string",
            Value::Char(_)            => "char",
            Value::List(_)            => "list",
            Value::Dict(_)            => "dict",
            Value::Tuple(_)           => "tuple",
            Value::Range { .. }       => "range",
            Value::Object { .. }      => "object",
            Value::Enum { .. }        => "enum",
            Value::Function(_)        => "function",
            Value::NativeFunction{..} => "native_function",
            Value::None               => "none",
            Value::Return(_)          => "return",
            Value::Break(_)           => "break",
            Value::Continue           => "continue",
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Int(i)     => i.to_string(),
            Value::Float(f)   => {
                if f.fract() == 0.0 { format!("{:.1}", f) } else { f.to_string() }
            }
            Value::Bool(b)    => b.to_string(),
            Value::String(s)  => s.clone(),
            Value::Char(c)    => c.to_string(),
            Value::None       => "none".into(),
            Value::List(v)    => {
                let items: Vec<String> = v.iter().map(|x| x.repr()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Dict(m)    => {
                let items: Vec<String> = m.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.repr()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::Tuple(v)   => {
                let items: Vec<String> = v.iter().map(|x| x.repr()).collect();
                if items.len() == 1 { format!("({},)", items[0]) }
                else                { format!("({})", items.join(", ")) }
            }
            Value::Range { start, end, inclusive } => {
                if *inclusive { format!("{}..={}", start, end) }
                else          { format!("{}..{}", start, end) }
            }
            Value::Object { type_name, fields } => {
                let fs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.repr()))
                    .collect();
                format!("{} {{ {} }}", type_name, fs.join(", "))
            }
            Value::Enum { variant, fields, .. } => {
                if fields.is_empty() { variant.clone() }
                else {
                    let fs: Vec<String> = fields.iter().map(|v| v.repr()).collect();
                    format!("{}({})", variant, fs.join(", "))
                }
            }
            Value::Function(f)          => format!("<fn {}>", f.name),
            Value::NativeFunction{name,..} => format!("<builtin {}>", name),
            Value::Return(v)            => format!("<return {}>", v.display()),
            Value::Break(v)             => format!("<break {}>", v.display()),
            Value::Continue             => "<continue>".into(),
        }
    }

    /// Like display() but wraps strings in quotes for containers
    pub fn repr(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            Value::Char(c)   => format!("'{}'", c),
            other            => other.display(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b)   => *b,
            Value::Int(i)    => *i != 0,
            Value::Float(f)  => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::None      => false,
            Value::List(v)   => !v.is_empty(),
            _                => true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Environment (lexical scope chain)
// ─────────────────────────────────────────────────────────────────────────────
/// Scope maps are shared (`Arc<RwLock>`) rather than deep-copied: a function's
/// captured environment must see bindings defined *after* it (later top-level
/// `fn`s, sibling methods, imports), and cloning an env must stay O(1) —
/// closures contain envs which contain closures, so deep clones explode
/// exponentially.
#[derive(Clone)]
pub struct Env {
    vars:   Arc<std::sync::RwLock<HashMap<String, Value>>>,
    parent: Option<Arc<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: Arc::new(std::sync::RwLock::new(HashMap::new())), parent: None }
    }

    pub fn child(parent: Env) -> Self {
        Env { vars: Arc::new(std::sync::RwLock::new(HashMap::new())), parent: Some(Arc::new(parent)) }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.write().unwrap().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.read().unwrap().get(name) { return Some(v.clone()); }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    pub fn set(&self, name: &str, value: Value) -> bool {
        if self.vars.read().unwrap().contains_key(name) {
            self.vars.write().unwrap().insert(name.to_string(), value);
            return true;
        }
        if let Some(p) = &self.parent { return p.set(name, value); }
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Interpreter
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct Interpreter {
    pub env:       Env,
    call_depth:    usize,
    trait_registry: TraitRegistry,
    /// After a `Type::method` call that received `this`, holds the receiver's
    /// (possibly mutated) post-call value so the caller can write it back.
    last_this:     Option<Value>,
}

const MAX_CALL_DEPTH: usize = 512;

impl Interpreter {
    pub fn new() -> Self {
        let mut interp = Interpreter {
            env: Env::new(),
            call_depth: 0,
            trait_registry: TraitRegistry::new(),
            last_this: None,
        };
        interp.register_builtins();
        interp
    }

    pub fn interpret(&mut self, program: &Program) -> Result<()> {
        self.exec_block(&program.statements)?;
        // Invoke `main` if the program defined it (entry point convention)
        if let Some(f @ Value::Function(_)) = self.env.get("main") {
            self.call_value(f, vec![])?;
        }
        Ok(())
    }

    // ── public glue used by the module system, traits, and the compiler ──────

    /// Execute a single statement.
    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value> {
        self.exec_stmt(stmt)
    }

    /// Look up a value in the current environment.
    pub fn get_value(&self, name: &str) -> Option<Value> {
        self.env.get(name)
    }

    /// Snapshot of the current environment.
    pub fn get_environment(&self) -> Env {
        self.env.clone()
    }

    /// Replace the current environment.
    pub fn set_environment(&mut self, env: Env) {
        self.env = env;
    }

    /// Create a fresh child environment, switch to it, and return it.
    /// Used when executing module code so top-level names stay scoped.
    pub fn create_module_environment(&mut self) -> Env {
        let env = Env::child(self.env.clone());
        self.env = env.clone();
        env
    }

    /// Access to the trait registry.
    pub fn traits(&mut self) -> &mut TraitRegistry {
        &mut self.trait_registry
    }

    /// Run a program with async syntax support.
    ///
    /// `async`/`await` evaluate synchronously in the tree-walking interpreter —
    /// futures resolve immediately rather than being scheduled. This keeps
    /// async programs correct (if not concurrent) without a runtime dependency.
    pub fn execute_async(&mut self, program: &Program) -> Result<()> {
        self.interpret(program)
    }

    /// Load a module (stdlib or file-based) into the current environment.
    /// The module's exports are bound as `name::item` and as a `name` dict.
    pub fn load_module(&mut self, path: &str) -> Result<()> {
        let name = normalize_module_path(path);
        let exports = self.module_exports(&name)?;
        self.bind_module(&name, exports);
        Ok(())
    }

    /// Resolve a module path to its exports.
    pub fn module_exports(&mut self, name: &str) -> Result<HashMap<String, Value>> {
        let mut registry = ModuleRegistry::new();
        registry.import_module(name, self)
    }

    /// Bind a module's exports into the environment.
    fn bind_module(&mut self, name: &str, exports: HashMap<String, Value>) {
        let binding = name.rsplit('.').next().unwrap_or(name).to_string();
        let mut dict = HashMap::new();
        for (k, v) in exports {
            dict.insert(k.clone(), v.clone());
            self.env.define(format!("{}::{}", binding, k), v);
        }
        self.env.define(binding, Value::Dict(dict));
    }

    // ── block execution ───────────────────────────────────────────────────────

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<Value> {
        let mut last = Value::None;
        for stmt in stmts {
            let v = self.exec_stmt(stmt)?;
            match &v {
                Value::Return(_) | Value::Break(_) | Value::Continue => return Ok(v),
                _ => last = v,
            }
        }
        Ok(last)
    }

    /// Push a new lexical scope whose parent is the *actual* current env
    /// (not a clone), so writes to outer variables persist.
    fn push_scope(&mut self) {
        let parent = std::mem::replace(&mut self.env, Env::new());
        self.env.parent = Some(Arc::new(parent));
    }

    /// Pop the innermost scope, restoring its parent.
    fn pop_scope(&mut self) {
        if let Some(parent) = self.env.parent.take() {
            self.env = (*parent).clone();
        }
    }

    fn exec_block_scoped(&mut self, stmts: &[Stmt]) -> Result<Value> {
        self.push_scope();
        let result = self.exec_block(stmts);
        self.pop_scope();
        result
    }

    // ═════════════════════════════════════════════════════════════════════════
    // STATEMENT EXECUTION
    // ═════════════════════════════════════════════════════════════════════════

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Value> {
        match stmt {
            Stmt::Expr(e) => self.eval(e),

            Stmt::Let { name, value, mutable: _, .. } => {
                let v = self.eval(value)?;
                self.env.define(name.clone(), v);
                Ok(Value::None)
            }

            Stmt::Return(opt) => {
                let v = match opt {
                    Some(e) => match self.eval(e)? {
                        r @ Value::Return(_) => return Ok(r),
                        v => v,
                    },
                    None    => Value::None,
                };
                Ok(Value::Return(Box::new(v)))
            }

            Stmt::Break(opt) => {
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None    => Value::None,
                };
                Ok(Value::Break(Box::new(v)))
            }

            Stmt::Continue => Ok(Value::Continue),

            Stmt::If { condition, then_block, else_block } => {
                let cond = self.eval(condition)?;
                if cond.is_truthy() {
                    self.exec_block_scoped(then_block)
                } else if let Some(eb) = else_block {
                    self.exec_block_scoped(eb)
                } else {
                    Ok(Value::None)
                }
            }

            Stmt::While { condition, body, .. } => {
                loop {
                    let c = self.eval(condition)?;
                    if !c.is_truthy() { break; }
                    match self.exec_block_scoped(body)? {
                        Value::Break(_) => break,
                        Value::Return(v) => return Ok(Value::Return(v)),
                        _ => {}
                    }
                }
                Ok(Value::None)
            }

            Stmt::Loop { body, .. } => {
                loop {
                    match self.exec_block_scoped(body)? {
                        Value::Break(v) => return Ok(*v),
                        Value::Return(v) => return Ok(Value::Return(v)),
                        _ => {}
                    }
                }
            }

            Stmt::For { var, iterator, body, .. } => {
                let iter_val = self.eval(iterator)?;
                let items = self.to_iter(iter_val)?;
                for item in items {
                    self.push_scope();
                    self.env.define(var.clone(), item);
                    let result = self.exec_block(body)?;
                    self.pop_scope();
                    match result {
                        Value::Break(_) => break,
                        Value::Return(v) => return Ok(Value::Return(v)),
                        _ => {}
                    }
                }
                Ok(Value::None)
            }

            Stmt::Match { expr, arms } => {
                let v = self.eval(expr)?;
                for (pattern, guard, body) in arms {
                    self.push_scope();
                    let matched = self.match_pattern(&v, pattern)?;
                    // A guard that fails at runtime (e.g. `n > 0` on a string)
                    // simply skips the arm — guards do dynamic type filtering.
                    let proceed = matched && match guard {
                        Some(g) => self.eval(g).map(|v| v.is_truthy()).unwrap_or(false),
                        None    => true,
                    };
                    if proceed {
                        let r = self.exec_block(body)?;
                        self.pop_scope();
                        return Ok(r);
                    }
                    self.pop_scope();
                }
                Err(BruteError::PatternMatchError(v.display()))
            }

            Stmt::Function { name, params, body, is_async, .. } => {
                let f = Function {
                    name:     name.clone(),
                    params:   params.clone(),
                    body:     body.clone(),
                    closure:  Arc::new(self.env.clone()),
                    is_async: *is_async,
                };
                self.env.define(name.clone(), Value::Function(f));
                Ok(Value::None)
            }

            Stmt::Struct { name, fields: _, methods, .. } => {
                // Register a native constructor function (struct literals are
                // created via `Name { field: value }` syntax)
                self.env.define(name.clone(), Value::NativeFunction {
                    name: name.clone(),
                    func: |_interp, _args| {
                        Err(BruteError::RuntimeError(
                            "Use struct literal syntax: StructName { field: value }".into()
                        ))
                    },
                });

                // Register methods as `TypeName::method` in the *current* env so
                // `obj.method()` and `Type::method(...)` both resolve.
                for method in methods {
                    if let Stmt::Function { name: mname, params, body, is_async, .. } = method {
                        let f = Function {
                            name:     format!("{}::{}", name, mname),
                            params:   params.clone(),
                            body:     body.clone(),
                            closure:  Arc::new(self.env.clone()),
                            is_async: *is_async,
                        };
                        self.env.define(format!("{}::{}", name, mname), Value::Function(f));
                    }
                }
                Ok(Value::None)
            }

            Stmt::Enum { name, methods, .. } => {
                // Marker so `Enum::Variant` / `Enum::Variant(args)` can be
                // recognised at the call site (see Expr::PathAccess / Expr::Call).
                self.env.define(name.clone(), Value::String(format!("<enum {}>", name)));
                self.env.define(format!("enum:{}", name), Value::Bool(true));
                // Enum methods are registered as `Enum::method`, same as
                // struct/impl methods.
                for method in methods {
                    if let Stmt::Function { name: mname, params, body, is_async, .. } = method {
                        let f = Function {
                            name:     format!("{}::{}", name, mname),
                            params:   params.clone(),
                            body:     body.clone(),
                            closure:  Arc::new(self.env.clone()),
                            is_async: *is_async,
                        };
                        self.env.define(format!("{}::{}", name, mname), Value::Function(f));
                    }
                }
                Ok(Value::None)
            }

            Stmt::Const { name, value, .. } => {
                let v = self.eval(value)?;
                self.env.define(name.clone(), v);
                Ok(Value::None)
            }

            Stmt::Trait { name, methods, generic_params, .. } => {
                let t = self.trait_registry.create_trait_from_ast(name, methods, generic_params);
                self.trait_registry.register_trait(t);
                // Register default method bodies as `Trait::method` functions
                for m in methods {
                    if let Some(body) = &m.body {
                        let f = Function {
                            name:     format!("{}::{}", name, m.name),
                            params:   m.params.clone(),
                            body:     body.clone(),
                            closure:  Arc::new(self.env.clone()),
                            is_async: m.is_async,
                        };
                        self.env.define(format!("{}::{}", name, m.name), Value::Function(f));
                    }
                }
                Ok(Value::None)
            }

            Stmt::Impl { trait_name, type_name, methods, generic_params } => {
                // `type_name` may carry generic args (`Box<int>`) — register
                // methods under both the specialised and the base type name so
                // `Box<int>::m` and `Box::m` both resolve.
                let base = type_name.split('<').next().unwrap_or(type_name).to_string();
                for method in methods {
                    if let Stmt::Function { name: mname, params, body, is_async, .. } = method {
                        let f = Function {
                            name:     format!("{}::{}", type_name, mname),
                            params:   params.clone(),
                            body:     body.clone(),
                            closure:  Arc::new(self.env.clone()),
                            is_async: *is_async,
                        };
                        let fv = Value::Function(f);
                        self.env.define(format!("{}::{}", type_name, mname), fv.clone());
                        if base != *type_name {
                            self.env.define(format!("{}::{}", base, mname), fv);
                        }
                    }
                }

                // Record the impl in the trait registry when it names a trait
                if let Some(tn) = trait_name {
                    if self.trait_registry.has_trait(tn) {
                        let mut impl_methods = HashMap::new();
                        for method in methods {
                            if let Stmt::Function { name: mname, .. } = method {
                                if let Some(v) = self.env.get(&format!("{}::{}", type_name, mname)) {
                                    impl_methods.insert(mname.clone(), v);
                                }
                            }
                        }
                        let _ = self.trait_registry.register_impl(crate::traits::TraitImpl {
                            trait_name: tn.clone(),
                            type_name: type_name.clone(),
                            methods: impl_methods,
                            generic_params: generic_params.clone(),
                        });
                    }
                }
                Ok(Value::None)
            }

            Stmt::Use { path, as_name } => {
                let norm = normalize_module_path(path);
                // Try the whole path as a module first; otherwise treat the
                // last segment as an item exported by the parent module.
                match self.module_exports(&norm) {
                    Ok(exports) => {
                        if let Some(alias) = as_name {
                            self.env.define(alias.clone(), Value::Dict(exports));
                        } else {
                            self.bind_module(&norm, exports);
                        }
                        Ok(Value::None)
                    }
                    Err(_) => {
                        let (mod_part, item) = norm.rsplit_once('.')
                            .ok_or_else(|| BruteError::ModuleNotFound(norm.clone()))?;
                        let exports = self.module_exports(mod_part)?;
                        let v = exports.get(item).cloned().ok_or_else(|| {
                            BruteError::ModuleError(format!(
                                "module '{}' has no export '{}'", mod_part, item))
                        })?;
                        self.env.define(as_name.clone().unwrap_or_else(|| item.to_string()), v);
                        Ok(Value::None)
                    }
                }
            }

            Stmt::Import { path, items } => {
                let norm = normalize_module_path(path);
                let exports = self.module_exports(&norm)?;
                for item in items {
                    match item {
                        ImportItem::All => {
                            let binding = norm.rsplit('.').next().unwrap_or(&norm).to_string();
                            for (k, v) in &exports {
                                self.env.define(k.clone(), v.clone());
                            }
                            self.env.define(binding, Value::Dict(exports.clone()));
                        }
                        ImportItem::Name(n) => {
                            let v = exports.get(n).cloned().ok_or_else(|| {
                                BruteError::ModuleError(format!(
                                    "module '{}' has no export '{}'", norm, n))
                            })?;
                            self.env.define(n.clone(), v);
                        }
                        ImportItem::Alias(orig, alias) => {
                            let v = exports.get(orig).cloned().ok_or_else(|| {
                                BruteError::ModuleError(format!(
                                    "module '{}' has no export '{}'", norm, orig))
                            })?;
                            self.env.define(alias.clone(), v);
                        }
                    }
                }
                Ok(Value::None)
            }

            Stmt::TypeAlias { .. } => Ok(Value::None),

            Stmt::Try { block, catch_blocks, finally_block } => {
                let result = self.exec_block_scoped(block);
                let outcome = match result {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        let mut handled = false;
                        let mut handler_result = Ok(Value::None);
                        for catch in catch_blocks {
                            self.push_scope();
                            if let Some(binding) = &catch.binding {
                                self.env.define(binding.clone(), Value::String(e.to_string()));
                            }
                            handler_result = self.exec_block(&catch.body);
                            self.pop_scope();
                            handled = true;
                            break;
                        }
                        if handled { handler_result } else { Err(e) }
                    }
                };
                if let Some(fin) = finally_block {
                    self.exec_block_scoped(fin)?;
                }
                outcome
            }

            Stmt::Async { block } => self.exec_block_scoped(block),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // EXPRESSION EVALUATION
    // ═════════════════════════════════════════════════════════════════════════

    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Ok(self.eval_literal(lit)),

            Expr::Identifier(name) => {
                self.env.get(name)
                    .ok_or_else(|| BruteError::UndefinedVariable(name.clone()))
            }

            Expr::PathAccess { path } => {
                let joined = path.join("::");
                if let Some(v) = self.env.get(&joined) { return Ok(v); }

                // Longest-prefix resolution: `A::b::c` may mean "env binding
                // `A` (a dict/module/object), then field `b`, then `c`".
                for i in (1..path.len()).rev() {
                    if let Some(mut cur) = self.env.get(&path[..i].join("::")) {
                        let mut ok = true;
                        for seg in &path[i..] {
                            cur = match cur {
                                Value::Dict(m)   => m.get(seg).cloned(),
                                Value::Object { fields, .. } => fields.get(seg).cloned(),
                                _ => None,
                            }.unwrap_or_else(|| { ok = false; Value::None });
                            if !ok { break; }
                        }
                        if ok { return Ok(cur); }
                    }
                }

                // Unit enum variant: `Result::Err` where `enum:Result` is defined
                if path.len() >= 2 {
                    let base = path[..path.len()-1].join("::");
                    if matches!(self.env.get(&format!("enum:{}", base)), Some(Value::Bool(true))) {
                        return Ok(Value::Enum {
                            type_name: base,
                            variant:   path.last().unwrap().clone(),
                            fields:    vec![],
                        });
                    }
                }

                self.env.get(&path[0])
                    .ok_or_else(|| BruteError::UndefinedVariable(joined))
            }

            Expr::Assign { target, value } => {
                let v = self.eval(value)?;
                self.assign_target(target, v.clone())?;
                Ok(v)
            }

            Expr::CompoundAssign { target, op, value } => {
                let current = self.eval(target)?;
                let rhs     = self.eval(value)?;
                let result  = self.apply_compound_op(op, current, rhs)?;
                self.assign_target(target, result.clone())?;
                Ok(result)
            }

            Expr::BinaryOp { left, op, right } => {
                // Short-circuit for logical ops
                match op {
                    BinOp::And => {
                        let l = self.eval(left)?;
                        if !l.is_truthy() { return Ok(Value::Bool(false)); }
                        let r = self.eval(right)?;
                        return Ok(Value::Bool(r.is_truthy()));
                    }
                    BinOp::Or => {
                        let l = self.eval(left)?;
                        if l.is_truthy() { return Ok(Value::Bool(true)); }
                        let r = self.eval(right)?;
                        return Ok(Value::Bool(r.is_truthy()));
                    }
                    BinOp::NullCoalesce => {
                        let l = self.eval(left)?;
                        return match l {
                            Value::None => self.eval(right),
                            other => Ok(other),
                        };
                    }
                    BinOp::Range => {
                        let l = self.eval(left)?;
                        let r = self.eval(right)?;
                        return match (l, r) {
                            (Value::Int(s), Value::Int(e)) =>
                                Ok(Value::Range { start: s, end: e, inclusive: false }),
                            _ => Err(BruteError::TypeError("Range requires integer bounds".into())),
                        };
                    }
                    BinOp::RangeInclusive => {
                        let l = self.eval(left)?;
                        let r = self.eval(right)?;
                        return match (l, r) {
                            (Value::Int(s), Value::Int(e)) =>
                                Ok(Value::Range { start: s, end: e, inclusive: true }),
                            _ => Err(BruteError::TypeError("Range requires integer bounds".into())),
                        };
                    }
                    BinOp::Is => {
                        // `value is TypeName` — the right side names a type.
                        let l = self.eval(left)?;
                        let want = match right.as_ref() {
                            Expr::Identifier(n) => n.clone(),
                            _ => return Err(BruteError::TypeError("'is' expects a type name".into())),
                        };
                        let actual = match &l {
                            Value::Object { type_name, .. } | Value::Enum { type_name, .. } => {
                                type_name.clone()
                            }
                            other => other.type_name().to_string(),
                        };
                        return Ok(Value::Bool(actual == want || l.type_name() == want));
                    }
                    BinOp::Pipeline => {
                        // `value |> f`, `value |> f(_, x)`, `value |> _.method()`,
                        // `value |> [expr for x in _]` — `_` names the piped value.
                        let lv = self.eval(left)?;
                        self.push_scope();
                        self.env.define("_".to_string(), lv.clone());
                        let result = match right.as_ref() {
                            Expr::Call { func, args } => {
                                let has_hole = args.iter().any(|a| {
                                    matches!(a, Expr::Identifier(n) if n == "_")
                                });
                                let callee = self.eval(func)?;
                                let mut vals = Vec::with_capacity(args.len() + 1);
                                if !has_hole { vals.push(lv.clone()); }
                                for a in args { vals.push(self.eval(a)?); }
                                self.call_value(callee, vals)
                            }
                            Expr::Identifier(_) => {
                                let callee = self.eval(right)?;
                                self.call_value(callee, vec![lv])
                            }
                            _ => {
                                // Arbitrary expression — `_` resolves to the
                                // piped value; if it evaluates to a callable
                                // (e.g. a lambda), call it with the value.
                                match self.eval(right)? {
                                    f @ (Value::Function(_) | Value::NativeFunction { .. }) => {
                                        self.call_value(f, vec![lv])
                                    }
                                    v => Ok(v),
                                }
                            }
                        };
                        self.pop_scope();
                        return result;
                    }
                    _ => {}
                }
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                self.apply_binop(op, l, r)
            }

            Expr::UnaryOp { op, expr } => {
                let v = self.eval(expr)?;
                match op {
                    UnaryOp::Neg => match v {
                        Value::Int(i)   => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        _ => Err(BruteError::TypeError(format!("Cannot negate {}", v.type_name()))),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                    UnaryOp::BitNot => match v {
                        Value::Int(i) => Ok(Value::Int(!i)),
                        _ => Err(BruteError::TypeError("Bitwise NOT requires int".into())),
                    },
                    UnaryOp::Deref => self.deref_value(v),
                }
            }

            Expr::Call { func, args } => {
                // Enum constructor: `Result::Ok(v)` where `enum:Result` is defined
                if let Expr::PathAccess { path } = func.as_ref() {
                    if path.len() >= 2 {
                        let base = path[..path.len()-1].join("::");
                        if matches!(self.env.get(&format!("enum:{}", base)), Some(Value::Bool(true))) {
                            let mut vals = Vec::with_capacity(args.len());
                            for a in args { vals.push(self.eval(a)?); }
                            return Ok(Value::Enum {
                                type_name: base,
                                variant:   path.last().unwrap().clone(),
                                fields:    vals,
                            });
                        }
                    }
                }
                let callee = self.eval(func)?;
                let mut vals = Vec::with_capacity(args.len());
                for a in args { vals.push(self.eval(a)?); }
                self.call_value(callee, vals)
            }

            Expr::MethodCall { object, method, args } => {
                // In-place mutation for mutating methods on variables or
                // fields: `list.push(x)`, `this.items.push(x)`, `dict.set(k,v)`
                if let Some(result) = self.try_mutating_method(object, method, args)? {
                    return Ok(result);
                }
                let obj = self.eval(object)?;
                let mut vals = Vec::with_capacity(args.len());
                for a in args { vals.push(self.eval(a)?); }
                self.last_this = None;
                let result = self.call_method(obj, method, vals)?;
                // If the callee mutated `this`, write it back to the receiver
                // expression (supports `obj.mutating()` and `this.field.m()`).
                if let Some(new_this) = self.last_this.take() {
                    let _ = self.assign_target(object, new_this);
                }
                Ok(result)
            }

            Expr::FieldAccess { object, field } => {
                let obj = self.eval(object)?;
                match obj {
                    Value::Object { fields, .. } => fields.get(field)
                        .cloned()
                        .ok_or_else(|| BruteError::RuntimeError(format!("Field '{}' not found", field))),
                    Value::Enum { fields, .. } => {
                        fields.get(0).cloned()
                            .ok_or_else(|| BruteError::RuntimeError(format!("Field '{}' not found", field)))
                    }
                    // Dict member access: `dict.key`, module dicts `io.println`
                    Value::Dict(m) => Ok(m.get(field).cloned().unwrap_or(Value::None)),
                    // Nil-safe chaining: `none.field` yields `none`
                    Value::None => Ok(Value::None),
                    _ => Err(BruteError::TypeError(format!("Cannot access field '{}' on {}", field, obj.type_name()))),
                }
            }

            Expr::Index { target, index } => {
                let t = self.eval(target)?;
                let i = self.eval(index)?;
                self.index_value(t, i)
            }

            Expr::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for i in items { v.push(self.eval(i)?); }
                Ok(Value::List(v))
            }

            Expr::Dict(pairs) => {
                let mut m = HashMap::new();
                for (k, v) in pairs {
                    let key = self.eval(k)?.display();
                    let val = self.eval(v)?;
                    m.insert(key, val);
                }
                Ok(Value::Dict(m))
            }

            Expr::Tuple(items) => {
                let mut v = Vec::with_capacity(items.len());
                for i in items { v.push(self.eval(i)?); }
                Ok(Value::Tuple(v))
            }

            Expr::StructInit { name, fields } => {
                let mut fmap = HashMap::new();
                for (fname, fexpr) in fields {
                    fmap.insert(fname.clone(), self.eval(fexpr)?);
                }
                Ok(Value::Object { type_name: name.clone(), fields: fmap })
            }

            Expr::Block { stmts, trailing } => {
                self.push_scope();
                let mut result = Value::None;
                for s in stmts {
                    result = self.exec_stmt(s)?;
                    if matches!(&result, Value::Return(_) | Value::Break(_) | Value::Continue) {
                        self.pop_scope();
                        return Ok(result);
                    }
                }
                if let Some(t) = trailing {
                    result = self.eval(t)?;
                }
                self.pop_scope();
                Ok(result)
            }

            Expr::If { condition, then_expr, else_expr } => {
                let c = self.eval(condition)?;
                if c.is_truthy() {
                    self.eval(then_expr)
                } else if let Some(e) = else_expr {
                    self.eval(e)
                } else {
                    Ok(Value::None)
                }
            }

            Expr::Match { expr, arms } => {
                let v = self.eval(expr)?;
                for arm in arms {
                    let saved = self.env.clone();
                    self.env = Env::child(saved.clone());
                    if self.match_pattern(&v, &arm.pattern)? {
                        let proceed = if let Some(g) = &arm.guard {
                            self.eval(g).map(|v| v.is_truthy()).unwrap_or(false)
                        } else { true };
                        if proceed {
                            let r = self.eval(&arm.body)?;
                            self.env = saved;
                            return Ok(r);
                        }
                    }
                    self.env = saved;
                }
                Err(BruteError::PatternMatchError(v.display()))
            }

            Expr::ListComprehension { expr, var_name, iterable, condition } => {
                let iter_val = self.eval(iterable)?;
                let items    = self.to_iter(iter_val)?;
                let mut result = Vec::new();
                for item in items {
                    let saved = self.env.clone();
                    self.env = Env::child(saved.clone());
                    self.env.define(var_name.clone(), item);
                    let include = if let Some(cond) = condition {
                        self.eval(cond)?.is_truthy()
                    } else { true };
                    if include {
                        result.push(self.eval(expr)?);
                    }
                    self.env = saved;
                }
                Ok(Value::List(result))
            }

            Expr::Lambda { params, return_type: _, body } => {
                let f = Function {
                    name:     "<lambda>".into(),
                    params:   params.iter().map(|(n, t)| Param {
                        name:    n.clone(),
                        ty:      t.clone().unwrap_or(Type::Custom("any".into())),
                        default: None,
                    }).collect(),
                    body:     vec![Stmt::Return(Some((**body).clone()))],
                    closure:  Arc::new(self.env.clone()),
                    is_async: false,
                };
                Ok(Value::Function(f))
            }

            Expr::Await { expr } => self.eval(expr), // simplified (no real async runtime)

            Expr::Try { expr } => {
                // ? operator: `Err` propagates, `Ok`/`Some` unwrap, `None`
                // yields `none` (nil-safe chaining continues downstream)
                match self.eval(expr)? {
                    Value::Enum { variant, mut fields, .. }
                        if variant.eq_ignore_ascii_case("err") =>
                    {
                        let msg = fields.first().map(|v| v.display()).unwrap_or_default();
                        Err(BruteError::RuntimeError(msg))
                    }
                    Value::Enum { variant, mut fields, .. }
                        if variant.eq_ignore_ascii_case("ok") || variant.eq_ignore_ascii_case("some") =>
                    {
                        Ok(fields.pop().unwrap_or(Value::None))
                    }
                    Value::Enum { variant, .. } if variant.eq_ignore_ascii_case("none") => {
                        Ok(Value::None)
                    }
                    other => Ok(other),
                }
            }

            Expr::TypeCast { expr, target_type } => {
                let v = self.eval(expr)?;
                self.cast_value(v, target_type)
            }

            Expr::Return(opt) => {
                let v = match opt {
                    Some(e) => match self.eval(e)? {
                        // already a control-flow value (e.g. `return` inside a block) — propagate
                        r @ Value::Return(_) => return Ok(r),
                        v => v,
                    },
                    None    => Value::None,
                };
                Ok(Value::Return(Box::new(v)))
            }

            Expr::Break(opt) => {
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None    => Value::None,
                };
                Ok(Value::Break(Box::new(v)))
            }

            Expr::Continue => Ok(Value::Continue),
        }
    }

    fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Int(i)    => Value::Int(*i),
            Literal::Float(f)  => Value::Float(*f),
            Literal::Bool(b)   => Value::Bool(*b),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c)   => Value::Char(*c),
            Literal::None      => Value::None,
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // OPERATOR HELPERS
    // ═════════════════════════════════════════════════════════════════════════

    fn apply_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value> {
        match op {
            BinOp::Add => match (l, r) {
                (Value::Int(a),    Value::Int(b))    => Ok(Value::Int(a.wrapping_add(b))),
                (Value::Float(a),  Value::Float(b))  => Ok(Value::Float(a + b)),
                (Value::Int(a),    Value::Float(b))  => Ok(Value::Float(a as f64 + b)),
                (Value::Float(a),  Value::Int(b))    => Ok(Value::Float(a + b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                (Value::String(a), b)                => Ok(Value::String(a + &b.display())),
                (a, Value::String(b))                => Ok(Value::String(a.display() + &b)),
                (Value::List(mut a), Value::List(b)) => { a.extend(b); Ok(Value::List(a)) }
                (l, r) => Err(BruteError::TypeError(format!("Cannot add {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Sub => match (l, r) {
                (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a.wrapping_sub(b))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a - b as f64)),
                // 'a' - '0' — digit/letter arithmetic
                (Value::Char(a),  Value::Char(b))  => Ok(Value::Int(a as i64 - b as i64)),
                (Value::Char(c),  Value::Int(i))   => Ok(Value::Int(c as i64 - i)),
                (Value::Int(i),   Value::Char(c))  => Ok(Value::Int(i - c as i64)),
                (l, r) => Err(BruteError::TypeError(format!("Cannot subtract {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Mul => match (l, r) {
                (Value::Int(a),    Value::Int(b))   => Ok(Value::Int(a.wrapping_mul(b))),
                (Value::Float(a),  Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a),    Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
                (Value::Float(a),  Value::Int(b))   => Ok(Value::Float(a * b as f64)),
                (Value::String(s), Value::Int(n))   => Ok(Value::String(s.repeat(n.max(0) as usize))),
                (Value::List(v),   Value::Int(n))   => {
                    let mut out = Vec::new();
                    for _ in 0..n.max(0) { out.extend(v.clone()); }
                    Ok(Value::List(out))
                }
                (l, r) => Err(BruteError::TypeError(format!("Cannot multiply {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Div => match (l, r) {
                (Value::Int(a),   Value::Int(b))   => { if b == 0 { return Err(BruteError::DivisionByZero); } Ok(Value::Int(a / b)) }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a),   Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b))   => { if b == 0 { return Err(BruteError::DivisionByZero); } Ok(Value::Float(a / b as f64)) }
                (l, r) => Err(BruteError::TypeError(format!("Cannot divide {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Mod => match (l, r) {
                (Value::Int(a),   Value::Int(b))   => { if b == 0 { return Err(BruteError::DivisionByZero); } Ok(Value::Int(a % b)) }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
                (l, r) => Err(BruteError::TypeError(format!("Cannot mod {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Pow => match (l, r) {
                (Value::Int(a),   Value::Int(b))   => Ok(Value::Int(a.wrapping_pow(b.max(0) as u32))),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
                (Value::Int(a),   Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
                (Value::Float(a), Value::Int(b))   => Ok(Value::Float(a.powi(b as i32))),
                (l, r) => Err(BruteError::TypeError(format!("Cannot exponentiate {} and {}", l.type_name(), r.type_name()))),
            },
            BinOp::Eq  => Ok(Value::Bool(self.values_equal(&l, &r))),
            BinOp::Ne  => Ok(Value::Bool(!self.values_equal(&l, &r))),
            BinOp::Lt  => self.compare(&l, &r).map(|o| Value::Bool(o < 0)),
            BinOp::Le  => self.compare(&l, &r).map(|o| Value::Bool(o <= 0)),
            BinOp::Gt  => self.compare(&l, &r).map(|o| Value::Bool(o > 0)),
            BinOp::Ge  => self.compare(&l, &r).map(|o| Value::Bool(o >= 0)),
            BinOp::BitAnd => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                _ => Err(BruteError::TypeError("Bitwise AND requires integers".into())),
            },
            BinOp::BitOr => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                _ => Err(BruteError::TypeError("Bitwise OR requires integers".into())),
            },
            BinOp::BitXor => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                _ => Err(BruteError::TypeError("Bitwise XOR requires integers".into())),
            },
            BinOp::Shl => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a << (b & 63))),
                _ => Err(BruteError::TypeError("Shift requires integers".into())),
            },
            BinOp::Shr => match (l, r) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a >> (b & 63))),
                _ => Err(BruteError::TypeError("Shift requires integers".into())),
            },
            BinOp::And | BinOp::Or | BinOp::NullCoalesce | BinOp::Is
            | BinOp::Pipeline | BinOp::Range | BinOp::RangeInclusive => unreachable!(),
        }
    }

    fn apply_compound_op(&self, op: &CompoundOp, l: Value, r: Value) -> Result<Value> {
        let binop = match op {
            CompoundOp::Add    => BinOp::Add,
            CompoundOp::Sub    => BinOp::Sub,
            CompoundOp::Mul    => BinOp::Mul,
            CompoundOp::Div    => BinOp::Div,
            CompoundOp::Mod    => BinOp::Mod,
            CompoundOp::Pow    => BinOp::Pow,
            CompoundOp::BitAnd => BinOp::BitAnd,
            CompoundOp::BitOr  => BinOp::BitOr,
            CompoundOp::BitXor => BinOp::BitXor,
        };
        self.apply_binop(&binop, l, r)
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a),    Value::Int(b))    => a == b,
            (Value::Float(a),  Value::Float(b))  => (a - b).abs() < f64::EPSILON,
            (Value::Int(a),    Value::Float(b))  => (*a as f64 - b).abs() < f64::EPSILON,
            (Value::Float(a),  Value::Int(b))    => (a - *b as f64).abs() < f64::EPSILON,
            (Value::Bool(a),   Value::Bool(b))   => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a),   Value::Char(b))   => a == b,
            (Value::None,      Value::None)      => true,
            // `Option::None` equals `none` — `x == none` and `none` patterns work
            (Value::None, Value::Enum { variant, .. })
            | (Value::Enum { variant, .. }, Value::None)
                if variant.eq_ignore_ascii_case("none") => true,
            (Value::Enum { type_name: t1, variant: v1, fields: f1 },
             Value::Enum { type_name: t2, variant: v2, fields: f2 }) => {
                t1 == t2 && v1 == v2
                    && f1.len() == f2.len()
                    && f1.iter().zip(f2.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            (Value::List(a),   Value::List(b))   => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            (Value::Tuple(a),  Value::Tuple(b))  => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| self.values_equal(x, y))
            }
            _ => false,
        }
    }

    pub fn compare(&self, a: &Value, b: &Value) -> Result<i32> {
        match (a, b) {
            (Value::Int(a),    Value::Int(b))    => Ok(a.cmp(b) as i32),
            (Value::Float(a),  Value::Float(b))  => Ok(a.partial_cmp(b).map(|o| o as i32).unwrap_or(0)),
            (Value::Int(a),    Value::Float(b))  => Ok((*a as f64).partial_cmp(b).map(|o| o as i32).unwrap_or(0)),
            (Value::Float(a),  Value::Int(b))    => Ok(a.partial_cmp(&(*b as f64)).map(|o| o as i32).unwrap_or(0)),
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b) as i32),
            (Value::Char(a),   Value::Char(b))   => Ok(a.cmp(b) as i32),
            _ => Err(BruteError::TypeError(
                format!("Cannot compare {} and {}", a.type_name(), b.type_name())
            )),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // CALLING
    // ═════════════════════════════════════════════════════════════════════════

    pub fn call_value(&mut self, callee: Value, args: Vec<Value>) -> Result<Value> {
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(BruteError::StackOverflow);
        }
        match callee {
            Value::NativeFunction { func, .. } => func(self, args),
            Value::Function(f) => self.call_function(f, args, None),
            // Dicts act as namespaces but may also be callable constructors:
            // `Timer()` resolves `Timer.create`/`Timer.new`/`Timer.call`
            Value::Dict(m) => {
                for key in ["call", "new", "create"] {
                    if let Some(f) = m.get(key).cloned() {
                        return self.call_value(f, args);
                    }
                }
                Err(BruteError::RuntimeError("'dict' is not callable".into()))
            }
            _ => Err(BruteError::RuntimeError(format!("'{}' is not callable", callee.type_name()))),
        }
    }

    /// Call `callee` as a method on `this`. Brute functions bind `this`
    /// (and a `this`/`self` parameter) to the receiver; native functions
    /// receive it as their first argument.
    pub fn call_value_method(&mut self, callee: Value, this: Value, args: Vec<Value>) -> Result<Value> {
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(BruteError::StackOverflow);
        }
        match callee {
            Value::Function(f) => self.call_function(f, args, Some(this)),
            Value::NativeFunction { func, .. } => {
                let mut all = Vec::with_capacity(args.len() + 1);
                all.push(this);
                all.extend(args);
                func(self, all)
            }
            _ => Err(BruteError::RuntimeError(format!("'{}' is not callable", callee.type_name()))),
        }
    }

    fn call_function(&mut self, f: Function, args: Vec<Value>, this: Option<Value>) -> Result<Value> {
        self.call_depth += 1;
        let saved = self.env.clone();
        self.env  = Env::child((*f.closure).clone());
        if let Some(t) = &this { self.env.define("this".into(), t.clone()); }
        // bind params; `this`/`self` params take the receiver, others consume args
        let mut ai = 0;
        for param in f.params.iter() {
            let val = if param.name == "self" || param.name == "this" {
                self.env.get("this").unwrap_or(Value::None)
            } else {
                let v = args.get(ai)
                    .cloned()
                    .or_else(|| param.default.as_ref().and_then(|d| self.eval(d).ok()))
                    .unwrap_or(Value::None);
                ai += 1;
                v
            };
            self.env.define(param.name.clone(), val);
        }
        let result = self.exec_block(&f.body);
        // If the method received a receiver, capture its post-call state so
        // `obj.mutating_method()` can propagate mutations back to `obj`.
        let new_this = if this.is_some() { self.env.get("this") } else { None };
        self.env = saved;
        self.last_this = new_this;
        self.call_depth -= 1;
        match result? {
            Value::Return(v) => Ok(*v),
            other            => Ok(other),
        }
    }

    /// Mutating methods (`push`, `insert`, `remove`, `set`, `clear` …) applied
    /// to a variable or object field mutate the binding in place.
    /// Returns `Ok(Some(result))` when the call was handled, `Ok(None)` to
    /// fall through to normal method dispatch.
    fn try_mutating_method(&mut self, object: &Expr, method: &str, args: &[Expr]) -> Result<Option<Value>> {
        match object {
            Expr::Identifier(var) => {
                let Some(container) = self.env.get(var) else { return Ok(None) };
                if let Some((new_container, result)) = self.mutating_op(&container, method, args)? {
                    self.env.set(var, new_container);
                    return Ok(Some(result));
                }
                Ok(None)
            }
            Expr::FieldAccess { object: inner, field } => {
                // `container.field.mutating(args)` — mutate the field of an
                // object stored in a variable (e.g. `this.items.push(x)`)
                let container = self.eval(inner)?;
                if let Value::Object { type_name, mut fields } = container {
                    if let Some(fv) = fields.get(field).cloned() {
                        if let Some((new_fv, result)) = self.mutating_op(&fv, method, args)? {
                            fields.insert(field.clone(), new_fv);
                            let new_obj = Value::Object { type_name, fields };
                            self.assign_target(inner, new_obj)?;
                            return Ok(Some(result));
                        }
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Apply a mutating list/dict operation to `container`, returning
    /// `Some((new_container, result))` — or `None` if not a mutating op.
    fn mutating_op(&mut self, container: &Value, method: &str, args: &[Expr]) -> Result<Option<(Value, Value)>> {
        match (container, method) {
            (Value::List(l), "push" | "append") => {
                let mut l = l.clone();
                let v = args.first().map(|e| self.eval(e)).transpose()?.unwrap_or(Value::None);
                l.push(v);
                // Builder-style: return the container so `v = v.push(x)` works.
                Ok(Some((Value::List(l.clone()), Value::List(l))))
            }
            (Value::List(l), "extend") => {
                let mut l = l.clone();
                if let Some(e) = args.first() {
                    let v = self.eval(e)?;
                    let items = self.to_iter(v)?;
                    l.extend(items);
                }
                Ok(Some((Value::List(l.clone()), Value::List(l))))
            }
            (Value::List(l), "insert") => {
                let mut l = l.clone();
                let i = match args.first().map(|e| self.eval(e)).transpose()? {
                    Some(Value::Int(i)) => if i < 0 { (l.len() as i64 + i).max(0) as usize } else { i as usize },
                    _ => l.len(),
                };
                let v = args.get(1).map(|e| self.eval(e)).transpose()?.unwrap_or(Value::None);
                l.insert(i.min(l.len()), v);
                Ok(Some((Value::List(l.clone()), Value::List(l))))
            }
            (Value::List(l), "remove_at" | "remove") => {
                let mut l = l.clone();
                let i = match args.first().map(|e| self.eval(e)).transpose()? {
                    Some(Value::Int(i)) => if i < 0 { l.len() as i64 + i } else { i },
                    _ => l.len() as i64 - 1,
                };
                if i < 0 || i as usize >= l.len() {
                    return Err(BruteError::IndexOutOfBounds { index: i, length: l.len() });
                }
                let removed = l.remove(i as usize);
                Ok(Some((Value::List(l), removed)))
            }
            (Value::List(l), "pop") => {
                let mut l = l.clone();
                let popped = l.pop().unwrap_or(Value::None);
                Ok(Some((Value::List(l), popped)))
            }
            (Value::List(_), "clear") => {
                let empty = Value::List(vec![]);
                Ok(Some((empty.clone(), empty)))
            }
            (Value::Dict(m), "set" | "insert") => {
                let mut m = m.clone();
                let k = args.first().map(|e| self.eval(e)).transpose()?
                    .map(|v| v.display()).unwrap_or_default();
                let v = args.get(1).map(|e| self.eval(e)).transpose()?.unwrap_or(Value::None);
                m.insert(k, v);
                Ok(Some((Value::Dict(m.clone()), Value::Dict(m))))
            }
            (Value::Dict(m), "remove") => {
                let mut m = m.clone();
                let k = args.first().map(|e| self.eval(e)).transpose()?
                    .map(|v| v.display()).unwrap_or_default();
                let removed = m.remove(&k).unwrap_or(Value::None);
                Ok(Some((Value::Dict(m), removed)))
            }
            (Value::Dict(_), "clear") => {
                let empty = Value::Dict(HashMap::new());
                Ok(Some((empty.clone(), empty)))
            }
            _ => Ok(None),
        }
    }

    fn call_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value> {
        // Nil-safe: `none.method()` propagates `none` — this makes
        // `opt?.method() ?? default` work for the `?` operator.
        if matches!(obj, Value::None) {
            return Ok(Value::None);
        }
        // Built-in methods per type
        match &obj {
            Value::String(s) => return self.string_method(s.clone(), method, args),
            Value::List(_)   => return self.list_method(obj, method, args),
            Value::Dict(m)   => {
                // Module dicts and plain dicts may store callables under keys —
                // `time.now()` resolves `time` (a dict) → `now` (a function).
                if let Some(f) = m.get(method).cloned() {
                    return self.call_value(f, args);
                }
                return self.dict_method(obj, method, args);
            }
            Value::Int(i)    => return self.int_method(*i, method, args),
            Value::Float(f)  => return self.float_method(*f, method, args),
            Value::Range { start, end, inclusive } => {
                return self.range_method(*start, *end, *inclusive, method, args);
            }
            _ => {}
        }
        // Generic: look up TypeName::method in env. Objects and enum values
        // report their declared type name (e.g. `Point`, `Result`).
        let type_name = match &obj {
            Value::Object { type_name, .. } | Value::Enum { type_name, .. } => type_name.clone(),
            other => other.type_name().to_string(),
        };
        // Specialised impls first: `Box<int>::method` before `Box::method`.
        // The generic arguments are inferred from the object's field types.
        if let Value::Object { fields, .. } = &obj {
            let mut field_types: Vec<(&String, &str)> = fields.iter()
                .map(|(n, v)| (n, v.type_name()))
                .collect();
            field_types.sort();
            if !field_types.is_empty() {
                let spec = format!("{}<{}>", type_name,
                    field_types.iter().map(|(_, t)| *t).collect::<Vec<_>>().join(","));
                if let Some(func) = self.env.get(&format!("{}::{}", spec, method)) {
                    return self.call_value_method(func, obj, args);
                }
                // Trait impls registered under the exact specialised name win
                // over the (possibly polluted) base `Type::method` binding.
                for imp in self.trait_registry.impls_for_exact(&spec) {
                    if let Some(func) = imp.methods.get(method).cloned() {
                        return self.call_value_method(func, obj, args);
                    }
                }
                // …then the defaults of the traits those impls implement.
                for imp in self.trait_registry.impls_for_exact(&spec) {
                    if let Some(func) = self.env.get(&format!("{}::{}", imp.trait_name, method)) {
                        return self.call_value_method(func, obj.clone(), args);
                    }
                }
            }
        }
        let key = format!("{}::{}", type_name, method);
        if let Some(func) = self.env.get(&key) {
            return self.call_value_method(func, obj, args);
        }
        // Trait impls whose registered type shares this base name
        // (`Collection<T>::add` serves a `Collection` object).
        for imp in self.trait_registry.impls_for_base(&type_name) {
            if let Some(func) = imp.methods.get(method).cloned() {
                return self.call_value_method(func, obj, args);
            }
        }
        // Trait default methods: if any trait is implemented for this type,
        // `Trait::method` (registered from the trait's default body) applies.
        for trait_name in self.trait_registry.traits_for_type(&type_name) {
            if let Some(func) = self.env.get(&format!("{}::{}", trait_name, method)) {
                return self.call_value_method(func, obj, args);
            }
        }
        // Built-in methods on enum values (Option/Result conveniences)
        if let Value::Enum { variant, fields, .. } = &obj {
            match method {
                "is_ok"    => return Ok(Value::Bool(variant.eq_ignore_ascii_case("ok"))),
                "is_err"   => return Ok(Value::Bool(variant.eq_ignore_ascii_case("err"))),
                "is_some"  => return Ok(Value::Bool(variant.eq_ignore_ascii_case("some"))),
                "is_none"  => return Ok(Value::Bool(variant.eq_ignore_ascii_case("none"))),
                "is_ready" => return Ok(Value::Bool(true)), // futures resolve eagerly
                "unwrap" | "expect" => {
                    if variant.eq_ignore_ascii_case("ok") || variant.eq_ignore_ascii_case("some") {
                        return Ok(fields.first().cloned().unwrap_or(Value::None));
                    }
                    let msg = args.first()
                        .map(|v| v.display())
                        .or_else(|| fields.first().map(|v| v.display()))
                        .unwrap_or_else(|| format!("Called {} on {}", method, variant));
                    return Err(BruteError::RuntimeError(msg));
                }
                "unwrap_or" => {
                    if variant.eq_ignore_ascii_case("ok") || variant.eq_ignore_ascii_case("some") {
                        return Ok(fields.first().cloned().unwrap_or(Value::None));
                    }
                    return Ok(args.first().cloned().unwrap_or(Value::None));
                }
                _ => {}
            }
        }
        if method == "is_ready" { return Ok(Value::Bool(true)); }
        // For objects: look up method in fields (native object methods get
        // the receiver as their first argument via call_value_method)
        if let Value::Object { ref fields, .. } = obj {
            if let Some(func) = fields.get(method).cloned() {
                return self.call_value_method(func, obj, args);
            }
        }
        // Universal methods available on every value
        match method {
            "to_string" | "to_str" | "display" => return Ok(Value::String(obj.display())),
            "clone" => return Ok(obj.clone()),
            "type_name" | "type_of" => return Ok(Value::String(obj.type_name().to_string())),
            _ => {}
        }
        Err(BruteError::RuntimeError(
            format!("Method '{}' not found on {}", method, obj.type_name())
        ))
    }

    /// `*v` — dereference handle objects (Mutex/Arc-style with `get`) to their
    /// inner value; any other value derefs to itself.
    fn deref_value(&mut self, v: Value) -> Result<Value> {
        if let Value::Object { fields, .. } = &v {
            if let Some(get @ Value::NativeFunction { .. }) = fields.get("get").cloned() {
                if fields.contains_key("__id") {
                    return self.call_value_method(get, v, vec![]);
                }
            }
        }
        Ok(v)
    }

    // ── assignment target ─────────────────────────────────────────────────────

    fn assign_target(&mut self, target: &Expr, value: Value) -> Result<()> {
        match target {
            // `*handle = v` / `*handle op= v` — write through to the shared
            // object's `set` method; otherwise assign the underlying target.
            Expr::UnaryOp { op: UnaryOp::Deref, expr } => {
                let inner = self.eval(expr)?;
                if let Value::Object { fields, .. } = &inner {
                    if let Some(set @ Value::NativeFunction { .. }) = fields.get("set").cloned() {
                        if fields.contains_key("__id") {
                            self.call_value_method(set, inner, vec![value])?;
                            return Ok(());
                        }
                    }
                }
                self.assign_target(expr, value)
            }
            Expr::Identifier(name) => {
                if !self.env.set(name, value.clone()) {
                    self.env.define(name.clone(), value);
                }
                Ok(())
            }
            Expr::Index { target: t, index } => {
                let idx = self.eval(index)?;
                let container_name = match t.as_ref() {
                    Expr::Identifier(n) => n.clone(),
                    _ => return Err(BruteError::RuntimeError("Complex index assignment not supported".into())),
                };
                let container = self.env.get(&container_name)
                    .ok_or_else(|| BruteError::UndefinedVariable(container_name.clone()))?;
                let new_container = match (container, idx) {
                    (Value::List(mut v), Value::Int(i)) => {
                        let i = if i < 0 { v.len() as i64 + i } else { i };
                        if i < 0 || i as usize >= v.len() {
                            return Err(BruteError::IndexOutOfBounds { index: i, length: v.len() });
                        }
                        v[i as usize] = value;
                        Value::List(v)
                    }
                    (Value::Dict(mut m), Value::String(k)) => {
                        m.insert(k, value);
                        Value::Dict(m)
                    }
                    _ => return Err(BruteError::RuntimeError("Invalid index assignment".into())),
                };
                self.env.set(&container_name, new_container);
                Ok(())
            }
            Expr::FieldAccess { object, field } => {
                let obj_name = match object.as_ref() {
                    Expr::Identifier(n) => n.clone(),
                    _ => return Err(BruteError::RuntimeError("Complex field assignment not supported".into())),
                };
                let obj = self.env.get(&obj_name)
                    .ok_or_else(|| BruteError::UndefinedVariable(obj_name.clone()))?;
                if let Value::Object { type_name, mut fields } = obj {
                    fields.insert(field.clone(), value);
                    self.env.set(&obj_name, Value::Object { type_name, fields });
                    Ok(())
                } else {
                    Err(BruteError::TypeError("Field assignment on non-object".into()))
                }
            }
            _ => Err(BruteError::RuntimeError("Invalid assignment target".into())),
        }
    }

    // ── indexing ──────────────────────────────────────────────────────────────

    fn index_value(&self, target: Value, index: Value) -> Result<Value> {
        match (target, index) {
            (Value::List(v), Value::Int(i)) => {
                let i = if i < 0 { v.len() as i64 + i } else { i };
                v.get(i as usize)
                    .cloned()
                    .ok_or_else(|| BruteError::IndexOutOfBounds { index: i, length: v.len() })
            }
            (Value::Tuple(v), Value::Int(i)) => {
                let i = if i < 0 { v.len() as i64 + i } else { i };
                v.get(i as usize)
                    .cloned()
                    .ok_or_else(|| BruteError::IndexOutOfBounds { index: i, length: v.len() })
            }
            (Value::String(s), Value::Int(i)) => {
                let chars: Vec<char> = s.chars().collect();
                let i = if i < 0 { chars.len() as i64 + i } else { i };
                chars.get(i as usize)
                    .map(|c| Value::Char(*c))
                    .ok_or_else(|| BruteError::IndexOutOfBounds { index: i, length: chars.len() })
            }
            (Value::Dict(m), Value::String(k)) => {
                m.get(&k).cloned().ok_or_else(|| BruteError::RuntimeError(format!("Key '{}' not found", k)))
            }
            _ => Err(BruteError::TypeError("Invalid index operation".into())),
        }
    }

    // ── type cast ─────────────────────────────────────────────────────────────

    fn cast_value(&self, v: Value, target: &Type) -> Result<Value> {
        match (v, target) {
            (Value::Int(i),    Type::Float)  => Ok(Value::Float(i as f64)),
            (Value::Float(f),  Type::Int)    => Ok(Value::Int(f as i64)),
            (Value::Int(i),    Type::String) => Ok(Value::String(i.to_string())),
            (Value::Float(f),  Type::String) => Ok(Value::String(f.to_string())),
            (Value::Bool(b),   Type::String) => Ok(Value::String(b.to_string())),
            (Value::String(s), Type::Int)    => s.trim().parse::<i64>()
                .map(Value::Int)
                .map_err(|_| BruteError::ValueError(format!("Cannot cast '{}' to int", s))),
            (Value::String(s), Type::Float)  => s.trim().parse::<f64>()
                .map(Value::Float)
                .map_err(|_| BruteError::ValueError(format!("Cannot cast '{}' to float", s))),
            (Value::String(s), Type::Bool)   => Ok(Value::Bool(!s.is_empty())),
            (Value::Int(i),    Type::Bool)   => Ok(Value::Bool(i != 0)),
            (Value::Float(f),  Type::Bool)   => Ok(Value::Bool(f != 0.0)),
            (Value::Char(c),   Type::Int)    => Ok(Value::Int(c as i64)),
            (Value::Int(i),    Type::Char)  => {
                char::from_u32(i as u32)
                    .map(Value::Char)
                    .ok_or_else(|| BruteError::ValueError(format!("Invalid char code {}", i)))
            }
            (v, _) => Ok(v), // identity cast for same types
        }
    }

    // ── iteration helper ──────────────────────────────────────────────────────

    pub fn to_iter(&self, v: Value) -> Result<Vec<Value>> {
        match v {
            Value::List(items)   => Ok(items),
            Value::Tuple(items)  => Ok(items),
            Value::String(s)     => Ok(s.chars().map(Value::Char).collect()),
            Value::Range { start, end, inclusive } => {
                let end_val = if inclusive { end + 1 } else { end };
                if start <= end_val {
                    Ok((start..end_val).map(Value::Int).collect())
                } else {
                    Ok((end_val..start).rev().map(Value::Int).collect())
                }
            }
            Value::Dict(m) => Ok(m.keys().map(|k| Value::String(k.clone())).collect()),
            _ => Err(BruteError::TypeError(format!("'{}' is not iterable", v.type_name()))),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // PATTERN MATCHING
    // ═════════════════════════════════════════════════════════════════════════

    fn match_pattern(&mut self, value: &Value, pattern: &Pattern) -> Result<bool> {
        match pattern {
            Pattern::Wildcard => Ok(true),

            Pattern::Rest => Ok(true),

            Pattern::Literal(lit) => {
                let pv = self.eval_literal(lit);
                Ok(self.values_equal(value, &pv))
            }

            Pattern::Identifier(name) => {
                // binding — always matches, captures value
                self.env.define(name.clone(), value.clone());
                Ok(true)
            }

            Pattern::Binding { name, pattern } => {
                if self.match_pattern(value, pattern)? {
                    self.env.define(name.clone(), value.clone());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            Pattern::Tuple(pats) => {
                if let Value::Tuple(items) = value {
                    if items.len() != pats.len() { return Ok(false); }
                    for (item, pat) in items.iter().zip(pats.iter()) {
                        if !self.match_pattern(item, pat)? { return Ok(false); }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            Pattern::List(pats) => {
                if let Value::List(items) = value {
                    // Trailing `..` or `..rest` / `...rest` — a named rest binds
                    // the remaining elements as a list.
                    let (has_rest, rest_name) = match pats.last() {
                        Some(Pattern::Rest) => (true, None),
                        Some(Pattern::Binding { name, pattern })
                            if matches!(**pattern, Pattern::Rest) => (true, Some(name.clone())),
                        _ => (false, None),
                    };
                    let fixed_pats = if has_rest { &pats[..pats.len()-1] } else { pats.as_slice() };
                    if !has_rest && items.len() != fixed_pats.len() { return Ok(false); }
                    if items.len() < fixed_pats.len() { return Ok(false); }
                    for (item, pat) in items.iter().zip(fixed_pats.iter()) {
                        if !self.match_pattern(item, pat)? { return Ok(false); }
                    }
                    if let Some(name) = rest_name {
                        self.env.define(name, Value::List(items[fixed_pats.len()..].to_vec()));
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            Pattern::Struct { name, fields, rest } => {
                match value {
                    Value::Object { type_name, fields: obj_fields } => {
                        if !name.is_empty() && type_name != name { return Ok(false); }
                        for (fname, fpat) in fields {
                            if let Some(fval) = obj_fields.get(fname) {
                                if !self.match_pattern(fval, fpat)? { return Ok(false); }
                            } else {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    // Anonymous `{ name, age }` patterns destructure dicts
                    Value::Dict(m) => {
                        if !name.is_empty() { return Ok(false); }
                        for (fname, fpat) in fields {
                            if let Some(fval) = m.get(fname) {
                                if !self.match_pattern(fval, fpat)? { return Ok(false); }
                            } else {
                                return Ok(false);
                            }
                        }
                        let _ = rest;
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }

            Pattern::EnumVariant { path, fields } => {
                let expected = path.last().cloned().unwrap_or_default();
                if let Value::Enum { variant, fields: enum_fields, .. } = value {
                    if !variant.eq_ignore_ascii_case(&expected) { return Ok(false); }
                    if fields.len() > enum_fields.len() { return Ok(false); }
                    for (field_val, field_pat) in enum_fields.iter().zip(fields.iter()) {
                        if !self.match_pattern(field_val, field_pat)? { return Ok(false); }
                    }
                    Ok(true)
                } else {
                    // `some(x)`/`ok(x)` match any non-`none` value and `none`
                    // matches `none` — lets `get()`-style methods that return
                    // raw values still destructure like Options.
                    if expected.eq_ignore_ascii_case("some") || expected.eq_ignore_ascii_case("ok") {
                        if matches!(value, Value::None) { return Ok(false); }
                        return match fields.first() {
                            Some(p) => self.match_pattern(value, p),
                            None    => Ok(true),
                        };
                    }
                    if expected.eq_ignore_ascii_case("none") {
                        return Ok(matches!(value, Value::None));
                    }
                    // Also match against plain identifiers used as enum variants
                    if let Value::String(s) = value {
                        Ok(*s == expected)
                    } else {
                        Ok(false)
                    }
                }
            }

            Pattern::Or(pats) => {
                for pat in pats {
                    // Save env state before trying each branch
                    let saved = self.env.clone();
                    if self.match_pattern(value, pat)? {
                        return Ok(true);
                    }
                    self.env = saved;
                }
                Ok(false)
            }

            Pattern::Range { start, end, inclusive } => {
                let sv = self.eval(start)?;
                let ev = self.eval(end)?;
                match (value, &sv, &ev) {
                    (Value::Int(v), Value::Int(s), Value::Int(e)) => {
                        Ok(if *inclusive { *v >= *s && *v <= *e } else { *v >= *s && *v < *e })
                    }
                    (Value::Char(v), Value::Char(s), Value::Char(e)) => {
                        Ok(if *inclusive { *v >= *s && *v <= *e } else { *v >= *s && *v < *e })
                    }
                    _ => Err(BruteError::TypeError("Range pattern type mismatch".into())),
                }
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // BUILT-IN METHODS  (String, List, Dict, Int, Float, Range)
    // ═════════════════════════════════════════════════════════════════════════

    fn string_method(&mut self, s: String, method: &str, args: Vec<Value>) -> Result<Value> {
        match method {
            "len" | "length"   => Ok(Value::Int(s.chars().count() as i64)),
            "to_string"        => Ok(Value::String(s)),
            "to_uppercase"     => Ok(Value::String(s.to_uppercase())),
            "to_lowercase"     => Ok(Value::String(s.to_lowercase())),
            "trim"             => Ok(Value::String(s.trim().to_string())),
            "trim_start"       => Ok(Value::String(s.trim_start().to_string())),
            "trim_end"         => Ok(Value::String(s.trim_end().to_string())),
            "is_empty"         => Ok(Value::Bool(s.is_empty())),
            "chars"            => Ok(Value::List(s.chars().map(Value::Char).collect())),
            "bytes"            => Ok(Value::List(s.bytes().map(|b| Value::Int(b as i64)).collect())),
            "lines"            => Ok(Value::List(s.lines().map(|l| Value::String(l.to_string())).collect())),
            "split" => {
                let sep = args.first().map(|v| v.display()).unwrap_or_default();
                Ok(Value::List(s.split(&*sep).map(|p| Value::String(p.to_string())).collect()))
            }
            "split_whitespace" => Ok(Value::List(s.split_whitespace().map(|p| Value::String(p.to_string())).collect())),
            "starts_with" => {
                let prefix = args.first().map(|v| v.display()).unwrap_or_default();
                Ok(Value::Bool(s.starts_with(&*prefix)))
            }
            "ends_with" => {
                let suffix = args.first().map(|v| v.display()).unwrap_or_default();
                Ok(Value::Bool(s.ends_with(&*suffix)))
            }
            "contains" => {
                let sub = args.first().map(|v| v.display()).unwrap_or_default();
                Ok(Value::Bool(s.contains(&*sub)))
            }
            "replace" => {
                let from = args.get(0).map(|v| v.display()).unwrap_or_default();
                let to   = args.get(1).map(|v| v.display()).unwrap_or_default();
                Ok(Value::String(s.replace(&*from, &*to)))
            }
            "find" => {
                let sub = args.first().map(|v| v.display()).unwrap_or_default();
                Ok(match s.find(&*sub) {
                    Some(i) => Value::Int(i as i64),
                    None    => Value::None,
                })
            }
            "slice" | "substring" => {
                let start = match args.get(0) { Some(Value::Int(i)) => *i as usize, _ => 0 };
                let end   = match args.get(1) { Some(Value::Int(i)) => *i as usize, _ => s.chars().count() };
                let chars: Vec<char> = s.chars().collect();
                let slice: String = chars[start.min(chars.len())..end.min(chars.len())].iter().collect();
                Ok(Value::String(slice))
            }
            "repeat" => {
                let n = match args.first() { Some(Value::Int(i)) => *i as usize, _ => 1 };
                Ok(Value::String(s.repeat(n)))
            }
            "parse_int" => s.trim().parse::<i64>().map(Value::Int)
                .map_err(|_| BruteError::ValueError(format!("Cannot parse '{}' as int", s))),
            "parse_float" => s.trim().parse::<f64>().map(Value::Float)
                .map_err(|_| BruteError::ValueError(format!("Cannot parse '{}' as float", s))),
            "to_int"   => s.trim().parse::<i64>().map(Value::Int)
                .map_err(|_| BruteError::ValueError(format!("Cannot parse '{}' as int", s))),
            "to_float" => s.trim().parse::<f64>().map(Value::Float)
                .map_err(|_| BruteError::ValueError(format!("Cannot parse '{}' as float", s))),
            "to_chars" => Ok(Value::List(s.chars().map(Value::Char).collect())),
            "reverse"  => Ok(Value::String(s.chars().rev().collect())),
            "join" => {
                // "sep".join(list)
                if let Some(Value::List(items)) = args.first() {
                    let parts: Vec<String> = items.iter().map(|v| v.display()).collect();
                    Ok(Value::String(parts.join(&s)))
                } else {
                    Err(BruteError::ArgumentError("join() expects a list".into()))
                }
            }
            _ => Err(BruteError::RuntimeError(format!("String has no method '{}'", method))),
        }
    }

    fn list_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value> {
        match method {
            "len" | "length" | "size" => {
                if let Value::List(v) = &obj { Ok(Value::Int(v.len() as i64)) }
                else { Ok(Value::Int(0)) }
            }
            "clone" => Ok(obj.clone()),
            "push" | "append" => {
                // Only reachable for non-identifier receivers (e.g. `[1,2].push(3)`);
                // `var.push(x)` mutates in place via try_mutating_method.
                if let Value::List(mut v) = obj {
                    if let Some(item) = args.into_iter().next() { v.push(item); }
                    Ok(Value::List(v))
                } else { Ok(obj) }
            }
            "get" => {
                if let Value::List(v) = &obj {
                    let i = match args.first() { Some(Value::Int(i)) => *i, _ => 0 };
                    let i = if i < 0 { v.len() as i64 + i } else { i };
                    Ok(v.get(i as usize).cloned().unwrap_or(Value::None))
                } else { Ok(Value::None) }
            }
            "pop" => {
                if let Value::List(mut v) = obj {
                    Ok(v.pop().unwrap_or(Value::None))
                } else { Ok(Value::None) }
            }
            "first" | "head" => {
                if let Value::List(v) = &obj { Ok(v.first().cloned().unwrap_or(Value::None)) }
                else { Ok(Value::None) }
            }
            "last" => {
                if let Value::List(v) = &obj { Ok(v.last().cloned().unwrap_or(Value::None)) }
                else { Ok(Value::None) }
            }
            "is_empty" => {
                if let Value::List(v) = &obj { Ok(Value::Bool(v.is_empty())) }
                else { Ok(Value::Bool(true)) }
            }
            "contains" => {
                if let Value::List(v) = &obj {
                    let needle = args.first().cloned().unwrap_or(Value::None);
                    Ok(Value::Bool(v.iter().any(|x| self.values_equal(x, &needle))))
                } else { Ok(Value::Bool(false)) }
            }
            "reverse" => {
                if let Value::List(mut v) = obj {
                    v.reverse();
                    Ok(Value::List(v))
                } else { Ok(obj) }
            }
            "sort" => {
                if let Value::List(mut v) = obj {
                    v.sort_by(|a, b| self.compare(a, b)
                        .map(|o| o.cmp(&0))
                        .unwrap_or(std::cmp::Ordering::Equal));
                    Ok(Value::List(v))
                } else { Ok(obj) }
            }
            "join" => {
                if let Value::List(v) = &obj {
                    let sep = args.first().map(|x| x.display()).unwrap_or_default();
                    let parts: Vec<String> = v.iter().map(|x| x.display()).collect();
                    Ok(Value::String(parts.join(&sep)))
                } else { Ok(Value::String(String::new())) }
            }
            "map" => {
                if let Value::List(v) = obj {
                    let func = args.into_iter().next().ok_or_else(|| BruteError::ArgumentError("map() needs a function".into()))?;
                    let mut result = Vec::new();
                    for item in v {
                        result.push(self.call_value(func.clone(), vec![item])?);
                    }
                    Ok(Value::List(result))
                } else { Ok(obj) }
            }
            "filter" => {
                if let Value::List(v) = obj {
                    let func = args.into_iter().next().ok_or_else(|| BruteError::ArgumentError("filter() needs a function".into()))?;
                    let mut result = Vec::new();
                    for item in v {
                        if self.call_value(func.clone(), vec![item.clone()])?.is_truthy() {
                            result.push(item);
                        }
                    }
                    Ok(Value::List(result))
                } else { Ok(obj) }
            }
            "reduce" | "fold" => {
                if let Value::List(v) = obj {
                    let func = args.get(0).cloned().ok_or_else(|| BruteError::ArgumentError("reduce() needs a function".into()))?;
                    let mut acc = args.get(1).cloned().or_else(|| v.first().cloned()).unwrap_or(Value::None);
                    let start  = if args.len() > 1 { 0 } else { 1 };
                    for item in v.into_iter().skip(start) {
                        acc = self.call_value(func.clone(), vec![acc, item])?;
                    }
                    Ok(acc)
                } else { Ok(Value::None) }
            }
            "slice" => {
                if let Value::List(v) = obj {
                    let start = match args.get(0) { Some(Value::Int(i)) => *i as usize, _ => 0 };
                    let end   = match args.get(1) { Some(Value::Int(i)) => *i as usize, _ => v.len() };
                    Ok(Value::List(v[start.min(v.len())..end.min(v.len())].to_vec()))
                } else { Ok(Value::List(vec![])) }
            }
            "flatten" => {
                if let Value::List(v) = obj {
                    let mut result = Vec::new();
                    for item in v {
                        if let Value::List(inner) = item { result.extend(inner); }
                        else { result.push(item); }
                    }
                    Ok(Value::List(result))
                } else { Ok(obj) }
            }
            "to_string" => {
                if let Value::List(v) = &obj { Ok(Value::String(obj.display())) }
                else { Ok(Value::String(obj.display())) }
            }
            "enumerate" => {
                if let Value::List(v) = obj {
                    Ok(Value::List(v.into_iter().enumerate()
                        .map(|(i, x)| Value::Tuple(vec![Value::Int(i as i64), x]))
                        .collect()))
                } else { Ok(Value::List(vec![])) }
            }
            _ => Err(BruteError::RuntimeError(format!("List has no method '{}'", method))),
        }
    }

    fn dict_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value> {
        match method {
            "len" | "length" => {
                if let Value::Dict(m) = &obj { Ok(Value::Int(m.len() as i64)) } else { Ok(Value::Int(0)) }
            }
            "keys" => {
                if let Value::Dict(m) = obj { Ok(Value::List(m.into_keys().map(Value::String).collect())) }
                else { Ok(Value::List(vec![])) }
            }
            "values" => {
                if let Value::Dict(m) = obj { Ok(Value::List(m.into_values().collect())) }
                else { Ok(Value::List(vec![])) }
            }
            "contains" | "has_key" | "contains_key" => {
                if let Value::Dict(m) = &obj {
                    let k = args.first().map(|v| v.display()).unwrap_or_default();
                    Ok(Value::Bool(m.contains_key(&k)))
                } else { Ok(Value::Bool(false)) }
            }
            "get" => {
                if let Value::Dict(m) = &obj {
                    let k = args.first().map(|v| v.display()).unwrap_or_default();
                    // `map.get(key, default)` — second arg is the fallback.
                    let default = args.get(1).cloned().unwrap_or(Value::None);
                    Ok(m.get(&k).cloned().unwrap_or(default))
                } else { Ok(Value::None) }
            }
            "is_empty" => {
                if let Value::Dict(m) = &obj { Ok(Value::Bool(m.is_empty())) } else { Ok(Value::Bool(true)) }
            }
            "to_string" => Ok(Value::String(obj.display())),
            _ => Err(BruteError::RuntimeError(format!("Dict has no method '{}'", method))),
        }
    }

    fn int_method(&self, i: i64, method: &str, args: Vec<Value>) -> Result<Value> {
        match method {
            "to_string"  => Ok(Value::String(i.to_string())),
            "to_float"   => Ok(Value::Float(i as f64)),
            "abs"        => Ok(Value::Int(i.abs())),
            "pow"        => {
                let exp = match args.first() { Some(Value::Int(n)) => *n as u32, _ => 1 };
                Ok(Value::Int(i.wrapping_pow(exp)))
            }
            "min"        => {
                let other = match args.first() { Some(Value::Int(n)) => *n, _ => i };
                Ok(Value::Int(i.min(other)))
            }
            "max"        => {
                let other = match args.first() { Some(Value::Int(n)) => *n, _ => i };
                Ok(Value::Int(i.max(other)))
            }
            "clamp"      => {
                let lo = match args.get(0) { Some(Value::Int(n)) => *n, _ => i64::MIN };
                let hi = match args.get(1) { Some(Value::Int(n)) => *n, _ => i64::MAX };
                Ok(Value::Int(i.clamp(lo, hi)))
            }
            "is_even"    => Ok(Value::Bool(i % 2 == 0)),
            "is_odd"     => Ok(Value::Bool(i % 2 != 0)),
            _            => Err(BruteError::RuntimeError(format!("Int has no method '{}'", method))),
        }
    }

    fn float_method(&self, f: f64, method: &str, args: Vec<Value>) -> Result<Value> {
        match method {
            "to_string"  => Ok(Value::String(f.to_string())),
            "to_int"     => Ok(Value::Int(f as i64)),
            "floor"      => Ok(Value::Float(f.floor())),
            "ceil"       => Ok(Value::Float(f.ceil())),
            "round"      => Ok(Value::Float(f.round())),
            "sqrt"       => Ok(Value::Float(f.sqrt())),
            "abs"        => Ok(Value::Float(f.abs())),
            "sin"        => Ok(Value::Float(f.sin())),
            "cos"        => Ok(Value::Float(f.cos())),
            "tan"        => Ok(Value::Float(f.tan())),
            "ln"         => Ok(Value::Float(f.ln())),
            "log2"       => Ok(Value::Float(f.log2())),
            "log10"      => Ok(Value::Float(f.log10())),
            "exp"        => Ok(Value::Float(f.exp())),
            "pow"        => {
                let exp = match args.first() {
                    Some(Value::Float(e)) => *e,
                    Some(Value::Int(e))   => *e as f64,
                    _ => 1.0,
                };
                Ok(Value::Float(f.powf(exp)))
            }
            "is_nan"     => Ok(Value::Bool(f.is_nan())),
            "is_finite"  => Ok(Value::Bool(f.is_finite())),
            "is_infinite"=> Ok(Value::Bool(f.is_infinite())),
            "min"        => {
                let other = match args.first() { Some(Value::Float(n)) => *n, Some(Value::Int(n)) => *n as f64, _ => f };
                Ok(Value::Float(f.min(other)))
            }
            "max"        => {
                let other = match args.first() { Some(Value::Float(n)) => *n, Some(Value::Int(n)) => *n as f64, _ => f };
                Ok(Value::Float(f.max(other)))
            }
            _ => Err(BruteError::RuntimeError(format!("Float has no method '{}'", method))),
        }
    }

    fn range_method(&self, start: i64, end: i64, inclusive: bool, method: &str, _args: Vec<Value>) -> Result<Value> {
        let end_val = if inclusive { end + 1 } else { end };
        match method {
            "len" | "count" => Ok(Value::Int((end_val - start).max(0))),
            "contains"      => Ok(Value::Bool(false)), // simplified
            "to_list"       => Ok(Value::List((start..end_val).map(Value::Int).collect())),
            "sum"           => Ok(Value::Int((start..end_val).sum())),
            _               => Err(BruteError::RuntimeError(format!("Range has no method '{}'", method))),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // BUILT-IN GLOBAL FUNCTIONS
    // ═════════════════════════════════════════════════════════════════════════

    fn register_builtins(&mut self) {
        macro_rules! builtin {
            ($name:expr, $func:expr) => {
                self.env.define($name.to_string(), Value::NativeFunction {
                    name: $name.to_string(),
                    func: $func,
                });
            };
        }

        // ── I/O ──────────────────────────────────────────────────────────────
        builtin!("println", |_interp, args| {
            let parts: Vec<String> = args.iter().map(|v| v.display()).collect();
            println!("{}", parts.join(" "));
            Ok(Value::None)
        });

        builtin!("print", |_interp, args| {
            let parts: Vec<String> = args.iter().map(|v| v.display()).collect();
            print!("{}", parts.join(" "));
            io::stdout().flush().ok();
            Ok(Value::None)
        });

        builtin!("eprintln", |_interp, args| {
            let parts: Vec<String> = args.iter().map(|v| v.display()).collect();
            eprintln!("{}", parts.join(" "));
            Ok(Value::None)
        });

        builtin!("input", |_interp, args| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt.display());
                io::stdout().flush().ok();
            }
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line).ok();
            Ok(Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
        });

        builtin!("read_line", |_interp, _args| {
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line).ok();
            Ok(Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()))
        });

        // ── type conversions ─────────────────────────────────────────────────
        builtin!("int", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Int(i)    => Ok(Value::Int(i)),
                Value::Float(f)  => Ok(Value::Int(f as i64)),
                Value::Bool(b)   => Ok(Value::Int(b as i64)),
                Value::String(s) => s.trim().parse::<i64>().map(Value::Int)
                    .map_err(|_| BruteError::ValueError(format!("Cannot convert '{}' to int", s))),
                Value::Char(c)   => Ok(Value::Int(c as i64)),
                other => Err(BruteError::TypeError(format!("Cannot convert {} to int", other.type_name()))),
            }
        });

        builtin!("float", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Float(f)  => Ok(Value::Float(f)),
                Value::Int(i)    => Ok(Value::Float(i as f64)),
                Value::Bool(b)   => Ok(Value::Float(b as i64 as f64)),
                Value::String(s) => s.trim().parse::<f64>().map(Value::Float)
                    .map_err(|_| BruteError::ValueError(format!("Cannot convert '{}' to float", s))),
                other => Err(BruteError::TypeError(format!("Cannot convert {} to float", other.type_name()))),
            }
        });

        builtin!("str", |_interp, args| {
            Ok(Value::String(args.into_iter().next().unwrap_or(Value::None).display()))
        });

        builtin!("bool", |_interp, args| {
            Ok(Value::Bool(args.into_iter().next().unwrap_or(Value::None).is_truthy()))
        });

        builtin!("char", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Int(i)  => char::from_u32(i as u32).map(Value::Char)
                    .ok_or_else(|| BruteError::ValueError(format!("Invalid char code {}", i))),
                Value::Char(c) => Ok(Value::Char(c)),
                other => Err(BruteError::TypeError(format!("Cannot convert {} to char", other.type_name()))),
            }
        });

        // ── introspection ─────────────────────────────────────────────────────
        builtin!("type_of", |_interp, args| {
            Ok(Value::String(args.into_iter().next().unwrap_or(Value::None).type_name().to_string()))
        });

        builtin!("is_none", |_interp, args| {
            Ok(Value::Bool(matches!(args.first(), Some(Value::None) | None)))
        });

        // ── collections ───────────────────────────────────────────────────────
        builtin!("len", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(v)   => Ok(Value::Int(v.len() as i64)),
                Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
                Value::Dict(m)   => Ok(Value::Int(m.len() as i64)),
                Value::Tuple(t)  => Ok(Value::Int(t.len() as i64)),
                Value::Range { start, end, inclusive } => {
                    let e = if inclusive { end + 1 } else { end };
                    Ok(Value::Int((e - start).max(0)))
                }
                other => Err(BruteError::TypeError(format!("len() cannot be applied to {}", other.type_name()))),
            }
        });

        builtin!("range", |_interp, args| {
            let (start, end, step) = match args.len() {
                1 => (0i64, match &args[0] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) }, 1i64),
                2 => (match &args[0] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) },
                      match &args[1] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) }, 1i64),
                _ => (match &args[0] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) },
                      match &args[1] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) },
                      match &args[2] { Value::Int(n) => *n, _ => return Err(BruteError::ArgumentError("range() needs int args".into())) }),
            };
            let mut v = Vec::new();
            let mut i = start;
            while if step > 0 { i < end } else { i > end } {
                v.push(Value::Int(i));
                i += step;
            }
            Ok(Value::List(v))
        });

        builtin!("enumerate", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(v) => Ok(Value::List(v.into_iter().enumerate()
                    .map(|(i, x)| Value::Tuple(vec![Value::Int(i as i64), x]))
                    .collect())),
                other => Err(BruteError::TypeError(format!("enumerate() requires a list, got {}", other.type_name()))),
            }
        });

        builtin!("zip", |_interp, args| {
            let lists: Vec<Vec<Value>> = args.into_iter().map(|a| match a {
                Value::List(v) => v,
                other => vec![other],
            }).collect();
            if lists.is_empty() { return Ok(Value::List(vec![])); }
            let min_len = lists.iter().map(|l| l.len()).min().unwrap_or(0);
            Ok(Value::List((0..min_len).map(|i| {
                Value::Tuple(lists.iter().map(|l| l[i].clone()).collect())
            }).collect()))
        });

        builtin!("push", |_interp, args| {
            // Functional push — returns new list
            let mut v = match args.get(0).cloned().unwrap_or(Value::None) {
                Value::List(v) => v,
                _ => return Err(BruteError::ArgumentError("push() needs a list as first arg".into())),
            };
            if let Some(item) = args.get(1).cloned() { v.push(item); }
            Ok(Value::List(v))
        });

        builtin!("pop", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(mut v) => { v.pop(); Ok(Value::List(v)) }
                other => Err(BruteError::ArgumentError(format!("pop() requires a list, got {}", other.type_name()))),
            }
        });

        builtin!("contains", |_interp, args| {
            match (args.get(0), args.get(1)) {
                (Some(Value::List(v)), Some(needle)) => {
                    Ok(Value::Bool(v.iter().any(|x| matches!((x, needle), (Value::Int(a), Value::Int(b)) if a == b
                        || matches!((x, needle), (Value::String(a), Value::String(b)) if a == b)))))
                }
                (Some(Value::String(s)), Some(Value::String(sub))) => Ok(Value::Bool(s.contains(sub.as_str()))),
                _ => Ok(Value::Bool(false)),
            }
        });

        // ── math ──────────────────────────────────────────────────────────────
        builtin!("abs", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Int(i)   => Ok(Value::Int(i.abs())),
                Value::Float(f) => Ok(Value::Float(f.abs())),
                other => Err(BruteError::TypeError(format!("abs() cannot be applied to {}", other.type_name()))),
            }
        });

        builtin!("min", |_interp, args| {
            args.into_iter().try_fold(Value::None, |acc, v| match acc {
                Value::None => Ok(v),
                other => match (&other, &v) {
                    (Value::Int(a), Value::Int(b))     => Ok(Value::Int(*a.min(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
                    _ => Ok(other),
                }
            })
        });

        builtin!("max", |_interp, args| {
            args.into_iter().try_fold(Value::None, |acc, v| match acc {
                Value::None => Ok(v),
                other => match (&other, &v) {
                    (Value::Int(a), Value::Int(b))     => Ok(Value::Int(*a.max(b))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
                    _ => Ok(other),
                }
            })
        });

        builtin!("sqrt", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Float(f) => Ok(Value::Float(f.sqrt())),
                Value::Int(i)   => Ok(Value::Float((i as f64).sqrt())),
                other => Err(BruteError::TypeError(format!("sqrt() cannot be applied to {}", other.type_name()))),
            }
        });

        builtin!("floor", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Float(f) => Ok(Value::Int(f.floor() as i64)),
                Value::Int(i)   => Ok(Value::Int(i)),
                other => Err(BruteError::TypeError(format!("floor() requires a number, got {}", other.type_name()))),
            }
        });

        builtin!("ceil", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Float(f) => Ok(Value::Int(f.ceil() as i64)),
                Value::Int(i)   => Ok(Value::Int(i)),
                other => Err(BruteError::TypeError(format!("ceil() requires a number, got {}", other.type_name()))),
            }
        });

        builtin!("round", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Float(f) => Ok(Value::Int(f.round() as i64)),
                Value::Int(i)   => Ok(Value::Int(i)),
                other => Err(BruteError::TypeError(format!("round() requires a number, got {}", other.type_name()))),
            }
        });

        builtin!("sum", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::List(v) => {
                    let mut total_int:   i64 = 0;
                    let mut total_float: f64 = 0.0;
                    let mut is_float = false;
                    for item in &v {
                        match item {
                            Value::Int(i)   => { total_int += i; total_float += *i as f64; }
                            Value::Float(f) => { is_float = true; total_float += f; }
                            _ => {}
                        }
                    }
                    if is_float { Ok(Value::Float(total_float)) } else { Ok(Value::Int(total_int)) }
                }
                other => Err(BruteError::TypeError(format!("sum() requires a list, got {}", other.type_name()))),
            }
        });

        // ── misc ──────────────────────────────────────────────────────────────
        builtin!("assert", |_interp, args| {
            let cond = args.first().cloned().unwrap_or(Value::None);
            if !cond.is_truthy() {
                let msg = args.get(1).map(|v| v.display()).unwrap_or_else(|| "Assertion failed".into());
                return Err(BruteError::RuntimeError(msg));
            }
            Ok(Value::None)
        });

        builtin!("panic", |_interp, args| {
            let msg = args.first().map(|v| v.display()).unwrap_or_else(|| "explicit panic".into());
            Err(BruteError::RuntimeError(msg))
        });

        builtin!("exit", |_interp, args| {
            let code = match args.first() { Some(Value::Int(i)) => *i as i32, _ => 0 };
            std::process::exit(code);
        });

        builtin!("repr", |_interp, args| {
            Ok(Value::String(args.first().cloned().unwrap_or(Value::None).repr()))
        });

        // ── Result/Option helpers ─────────────────────────────────────────────
        builtin!("ok", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Result".into(), variant: "Ok".into(), fields: vec![v] })
        });

        builtin!("err", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Result".into(), variant: "Err".into(), fields: vec![v] })
        });

        builtin!("some", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Option".into(), variant: "Some".into(), fields: vec![v] })
        });

        builtin!("none", |_interp, _args| {
            Ok(Value::Enum { type_name: "Option".into(), variant: "None".into(), fields: vec![] })
        });

        builtin!("unwrap", |_interp, args| {
            match args.into_iter().next().unwrap_or(Value::None) {
                Value::Enum { variant, fields, .. } if variant == "Some" || variant == "Ok" => {
                    Ok(fields.into_iter().next().unwrap_or(Value::None))
                }
                Value::Enum { variant, fields, .. } if variant == "None" || variant == "Err" => {
                    let msg = fields.into_iter().next().map(|v| v.display())
                        .unwrap_or_else(|| "Called unwrap on None/Err".into());
                    Err(BruteError::RuntimeError(msg))
                }
                other => Ok(other),
            }
        });

        // Capitalised aliases — `Ok(x)`, `Err(e)`, `Some(x)`, `None` values
        builtin!("Ok", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Result".into(), variant: "Ok".into(), fields: vec![v] })
        });
        builtin!("Err", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Result".into(), variant: "Err".into(), fields: vec![v] })
        });
        builtin!("Some", |_interp, args| {
            let v = args.into_iter().next().unwrap_or(Value::None);
            Ok(Value::Enum { type_name: "Option".into(), variant: "Some".into(), fields: vec![v] })
        });
        self.env.define("None".to_string(),
            Value::Enum { type_name: "Option".into(), variant: "None".into(), fields: vec![] });

        // ── string convenience functions (pipeline-friendly) ─────────────────
        builtin!("str_trim", |_interp, args| {
            Ok(Value::String(args.first().map(|v| v.display()).unwrap_or_default().trim().to_string()))
        });
        builtin!("str_uppercase", |_interp, args| {
            Ok(Value::String(args.first().map(|v| v.display()).unwrap_or_default().to_uppercase()))
        });
        builtin!("str_lowercase", |_interp, args| {
            Ok(Value::String(args.first().map(|v| v.display()).unwrap_or_default().to_lowercase()))
        });
        builtin!("str_reverse", |_interp, args| {
            Ok(Value::String(args.first().map(|v| v.display()).unwrap_or_default().chars().rev().collect()))
        });

        // ── Math constants ────────────────────────────────────────────────────
        self.env.define("PI".to_string(),  Value::Float(std::f64::consts::PI));
        self.env.define("E".to_string(),   Value::Float(std::f64::consts::E));
        self.env.define("TAU".to_string(), Value::Float(std::f64::consts::TAU));
        self.env.define("INF".to_string(), Value::Float(f64::INFINITY));
        self.env.define("NAN".to_string(), Value::Float(f64::NAN));
        self.env.define("INT_MAX".to_string(), Value::Int(i64::MAX));
        self.env.define("INT_MIN".to_string(), Value::Int(i64::MIN));
    }
} // end impl Interpreter

/// Normalise module paths written as `std.io`, `std::io`, `std/io`,
/// `stdlib::io`, or plain `io` into the canonical `io` form.
fn normalize_module_path(path: &str) -> String {
    let mut p = path.replace("::", ".").replace('/', ".").replace('\\', ".");
    for prefix in ["stdlib.", "std.", "stdlib"] {
        if let Some(rest) = p.strip_prefix(prefix) {
            p = rest.to_string();
            break;
        }
    }
    p.trim_matches('.').to_string()
}
