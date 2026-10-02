use std::collections::HashMap;
use crate::ast::{Type, Expr, Stmt, Literal, BinOp, Param};
use crate::error::{BruteError, Result};

// ─────────────────────────────────────────────────────────────────────────────
// Type environment
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct TypeEnv {
    vars:           HashMap<String, Type>,
    type_aliases:   HashMap<String, Type>,
    generic_params: Vec<String>,
    parent:         Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            vars:           HashMap::new(),
            type_aliases:   HashMap::new(),
            generic_params: Vec::new(),
            parent:         None,
        };
        // seed with built-in function signatures
        env.define_builtins();
        env
    }

    fn define_builtins(&mut self) {
        use Type::*;
        // Each built-in is typed as a Function so the checker can look it up.
        let builtins: &[(&str, Type)] = &[
            ("println",   Function(vec![Custom("any".into())], Box::new(Void))),
            ("print",     Function(vec![Custom("any".into())], Box::new(Void))),
            ("eprintln",  Function(vec![Custom("any".into())], Box::new(Void))),
            ("input",     Function(vec![Custom("any".into())], Box::new(String))),
            ("read_line", Function(vec![], Box::new(String))),
            ("len",       Function(vec![Custom("any".into())], Box::new(Int))),
            ("range",     Function(vec![Int], Box::new(List(Box::new(Int))))),
            ("enumerate", Function(vec![Custom("any".into())], Box::new(List(Box::new(Tuple(vec![])))))),
            ("zip",       Function(vec![Custom("any".into())], Box::new(List(Box::new(Tuple(vec![])))))),
            ("type_of",   Function(vec![Custom("any".into())], Box::new(String))),
            ("int",       Function(vec![Custom("any".into())], Box::new(Int))),
            ("float",     Function(vec![Custom("any".into())], Box::new(Float))),
            ("str",       Function(vec![Custom("any".into())], Box::new(String))),
            ("bool",      Function(vec![Custom("any".into())], Box::new(Bool))),
            ("abs",       Function(vec![Custom("any".into())], Box::new(Custom("any".into())))),
            ("min",       Function(vec![Custom("any".into())], Box::new(Custom("any".into())))),
            ("max",       Function(vec![Custom("any".into())], Box::new(Custom("any".into())))),
            ("sqrt",      Function(vec![Custom("any".into())], Box::new(Float))),
            ("sum",       Function(vec![List(Box::new(Custom("any".into())))], Box::new(Custom("any".into())))),
            ("assert",    Function(vec![Bool], Box::new(Void))),
            ("panic",     Function(vec![Custom("any".into())], Box::new(Never))),
            ("ok",        Function(vec![Custom("any".into())], Box::new(Result(Box::new(Custom("any".into())), Box::new(Custom("any".into())))))),
            ("err",       Function(vec![Custom("any".into())], Box::new(Result(Box::new(Custom("any".into())), Box::new(Custom("any".into())))))),
            ("some",      Function(vec![Custom("any".into())], Box::new(Option(Box::new(Custom("any".into())))))),
            ("unwrap",    Function(vec![Custom("any".into())], Box::new(Custom("any".into())))),
            ("push",      Function(vec![Custom("any".into()), Custom("any".into())], Box::new(List(Box::new(Custom("any".into())))))),
            ("pop",       Function(vec![Custom("any".into())], Box::new(List(Box::new(Custom("any".into())))))),
            ("repr",      Function(vec![Custom("any".into())], Box::new(String))),
        ];
        for (name, ty) in builtins {
            self.vars.insert((*name).to_string(), ty.clone());
        }
        // Constants
        self.vars.insert("PI".into(),      Type::Float);
        self.vars.insert("E".into(),       Type::Float);
        self.vars.insert("TAU".into(),     Type::Float);
        self.vars.insert("INF".into(),     Type::Float);
        self.vars.insert("NAN".into(),     Type::Float);
        self.vars.insert("INT_MAX".into(), Type::Int);
        self.vars.insert("INT_MIN".into(), Type::Int);
    }

    pub fn child(&self) -> Self {
        TypeEnv {
            vars:           HashMap::new(),
            type_aliases:   HashMap::new(),
            generic_params: Vec::new(),
            parent:         Some(Box::new(self.clone())),
        }
    }

    pub fn define(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<Type> {
        self.vars.get(name).cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    pub fn define_alias(&mut self, name: String, ty: Type) {
        self.type_aliases.insert(name, ty);
    }

    pub fn resolve_alias(&self, name: &str) -> Option<Type> {
        self.type_aliases.get(name).cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.resolve_alias(name)))
    }

    pub fn add_generic_params(&mut self, params: &[String]) {
        self.generic_params.extend_from_slice(params);
    }

    pub fn is_generic(&self, name: &str) -> bool {
        self.generic_params.contains(&name.to_string())
            || self.parent.as_ref().map(|p| p.is_generic(name)).unwrap_or(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Type checker
// ─────────────────────────────────────────────────────────────────────────────
pub struct TypeChecker {
    env:         TypeEnv,
    return_type: Option<Type>,
    errors:      Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            env:         TypeEnv::new(),
            return_type: None,
            errors:      Vec::new(),
        }
    }

    /// Run the type checker and return Ok(()) if there are no hard errors.
    pub fn check_program(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            if let Err(e) = self.check_stmt(stmt) {
                self.errors.push(e.to_string());
            }
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(BruteError::TypeError(self.errors.join("\n")))
        }
    }

    // ── statement checking ────────────────────────────────────────────────────

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, type_hint, value, .. } => {
                let inferred = self.infer(value)?;
                let ty = if let Some(hint) = type_hint {
                    if !self.compatible(hint, &inferred) {
                        return Err(BruteError::TypeError(format!(
                            "Type mismatch for '{}': declared {:?} but got {:?}", name, hint, inferred
                        )));
                    }
                    hint.clone()
                } else {
                    inferred
                };
                self.env.define(name.clone(), ty);
            }

            Stmt::Function { name, params, return_type, body, generic_params, is_async, .. } => {
                let old_env = self.env.clone();
                self.env = self.env.child();
                self.env.add_generic_params(generic_params);
                for p in params {
                    self.env.define(p.name.clone(), p.ty.clone());
                }
                let old_ret = self.return_type.replace(return_type.clone());
                for s in body { let _ = self.check_stmt(s); }
                self.return_type = old_ret;
                self.env = old_env;

                let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                let fn_type = Type::Function(param_types, Box::new(return_type.clone()));
                self.env.define(name.clone(), if *is_async {
                    Type::Future(Box::new(fn_type))
                } else {
                    fn_type
                });
            }

            Stmt::Struct { name, fields, generic_params, .. } => {
                let old_env = self.env.clone();
                self.env.add_generic_params(generic_params);
                for f in fields { let _ = self.check_type_valid(&f.ty); }
                self.env = old_env;
                self.env.define(name.clone(), Type::Custom(name.clone()));
            }

            Stmt::Enum { name, variants, generic_params, .. } => {
                let old_env = self.env.clone();
                self.env.add_generic_params(generic_params);
                self.env = old_env;
                self.env.define(name.clone(), Type::Custom(name.clone()));
            }

            Stmt::Const { name, type_hint, value, .. } => {
                let inferred = self.infer(value)?;
                if !self.compatible(type_hint, &inferred) {
                    return Err(BruteError::TypeError(format!(
                        "Constant '{}' type mismatch: declared {:?} but got {:?}", name, type_hint, inferred
                    )));
                }
                self.env.define(name.clone(), type_hint.clone());
            }

            Stmt::TypeAlias { name, alias_type, generic_params, .. } => {
                self.env.define_alias(name.clone(), alias_type.clone());
                self.env.define(name.clone(), alias_type.clone());
            }

            Stmt::Return(opt) => {
                if let Some(expr) = opt {
                    let _ty = self.infer(expr)?;
                    // Could check against self.return_type here
                }
            }

            Stmt::If { condition, then_block, else_block } => {
                let cty = self.infer(condition)?;
                // Warn but don't hard-fail; language supports truthy values
                let old = self.env.clone();
                self.env = self.env.child();
                for s in then_block { let _ = self.check_stmt(s); }
                self.env = old.clone();
                if let Some(eb) = else_block {
                    self.env = self.env.child();
                    for s in eb { let _ = self.check_stmt(s); }
                    self.env = old;
                }
            }

            Stmt::While { condition, body, .. } => {
                let _ = self.infer(condition)?;
                let old = self.env.clone();
                self.env = self.env.child();
                for s in body { let _ = self.check_stmt(s); }
                self.env = old;
            }

            Stmt::For { var, iterator, body, .. } => {
                let iter_ty = self.infer(iterator)?;
                let elem_ty = self.element_type(&iter_ty);
                let old = self.env.clone();
                self.env = self.env.child();
                self.env.define(var.clone(), elem_ty);
                for s in body { let _ = self.check_stmt(s); }
                self.env = old;
            }

            Stmt::Match { expr, arms } => {
                let _ = self.infer(expr)?;
                for (_, _, body) in arms {
                    let old = self.env.clone();
                    self.env = self.env.child();
                    for s in body { let _ = self.check_stmt(s); }
                    self.env = old;
                }
            }

            Stmt::Expr(e) => { let _ = self.infer(e); }

            // Everything else is either not type-checkable at this stage or is fine as-is.
            _ => {}
        }
        Ok(())
    }

    // ── expression type inference ─────────────────────────────────────────────

    pub fn infer(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),

            Expr::Identifier(name) => {
                self.env.get(name)
                    .ok_or_else(|| BruteError::TypeError(format!("Undefined variable '{}'", name)))
            }

            Expr::BinaryOp { left, op, right } => {
                let lt = self.infer(left)?;
                let rt = self.infer(right)?;
                self.infer_binop(op, &lt, &rt)
            }

            Expr::UnaryOp { op, expr } => {
                let ty = self.infer(expr)?;
                match op {
                    crate::ast::UnaryOp::Not    => Ok(Type::Bool),
                    crate::ast::UnaryOp::Neg    => Ok(ty),
                    crate::ast::UnaryOp::BitNot => Ok(Type::Int),
                    crate::ast::UnaryOp::Deref  => Ok(ty),
                }
            }

            Expr::Call { func, args } => {
                let fty = self.infer(func)?;
                match fty {
                    Type::Function(_, ret) => Ok(*ret),
                    Type::Future(inner)    => Ok(*inner),
                    Type::Custom(_)        => Ok(Type::Custom("any".into())),
                    _ => Ok(Type::Custom("any".into())),
                }
            }

            Expr::MethodCall { object, .. } => {
                // Simplified — return 'any' for method calls
                Ok(Type::Custom("any".into()))
            }

            Expr::FieldAccess { object, .. } => Ok(Type::Custom("any".into())),
            Expr::Index { .. }               => Ok(Type::Custom("any".into())),

            Expr::List(items) => {
                if items.is_empty() {
                    return Ok(Type::List(Box::new(Type::Custom("any".into()))));
                }
                let first = self.infer(&items[0])?;
                Ok(Type::List(Box::new(first)))
            }

            Expr::Tuple(items) => {
                let types: Result<Vec<_>> = items.iter().map(|e| self.infer(e)).collect();
                Ok(Type::Tuple(types?))
            }

            Expr::Dict(_) => Ok(Type::Dict(
                Box::new(Type::String),
                Box::new(Type::Custom("any".into())),
            )),

            Expr::Lambda { params, return_type, .. } => {
                let param_types = params.iter()
                    .map(|(_, t)| t.clone().unwrap_or(Type::Custom("any".into())))
                    .collect();
                let ret = return_type.clone().unwrap_or(Type::Custom("any".into()));
                Ok(Type::Function(param_types, Box::new(ret)))
            }

            Expr::If { then_expr, else_expr, .. } => {
                let t1 = self.infer(then_expr)?;
                if let Some(e) = else_expr {
                    let t2 = self.infer(e)?;
                    if self.compatible(&t1, &t2) { Ok(t1) } else { Ok(Type::Custom("any".into())) }
                } else {
                    Ok(Type::Option(Box::new(t1)))
                }
            }

            Expr::Match { arms, .. } => {
                if arms.is_empty() { return Ok(Type::Void); }
                let ty = self.infer(&arms[0].body)?;
                Ok(ty)
            }

            Expr::Block { stmts, trailing } => {
                let old = self.env.clone();
                self.env = self.env.child();
                for s in stmts { let _ = self.check_stmt(s); }
                let result = if let Some(t) = trailing {
                    self.infer(t)?
                } else {
                    Type::Void
                };
                self.env = old;
                Ok(result)
            }

            Expr::Await { expr }   => self.infer(expr),
            Expr::Try { expr }     => self.infer(expr),
            Expr::Return(_)        => Ok(Type::Never),
            Expr::Break(_)         => Ok(Type::Never),
            Expr::Continue         => Ok(Type::Never),

            Expr::TypeCast { target_type, .. } => Ok(target_type.clone()),

            Expr::StructInit { name, .. } => Ok(Type::Custom(name.clone())),

            Expr::ListComprehension { expr, .. } => {
                let elem = self.infer(expr)?;
                Ok(Type::List(Box::new(elem)))
            }

            Expr::Assign { value, .. }           => self.infer(value),
            Expr::CompoundAssign { value, .. }    => self.infer(value),
            Expr::PathAccess { .. }               => Ok(Type::Custom("any".into())),
        }
    }

    fn infer_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_)    => Type::Int,
            Literal::Float(_)  => Type::Float,
            Literal::Bool(_)   => Type::Bool,
            Literal::String(_) => Type::String,
            Literal::Char(_)   => Type::Char,
            Literal::None      => Type::Option(Box::new(Type::Custom("any".into()))),
        }
    }

    fn infer_binop(&self, op: &BinOp, lt: &Type, rt: &Type) -> Result<Type> {
        match op {
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
            | BinOp::And | BinOp::Or => Ok(Type::Bool),

            BinOp::Add => match (lt, rt) {
                (Type::String, _) | (_, Type::String) => Ok(Type::String),
                (Type::Float, _)  | (_, Type::Float)  => Ok(Type::Float),
                _                                      => Ok(Type::Int),
            },

            BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
                match (lt, rt) {
                    (Type::Float, _) | (_, Type::Float) => Ok(Type::Float),
                    _ => Ok(Type::Int),
                }
            }

            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => Ok(Type::Int),

            BinOp::Range | BinOp::RangeInclusive => Ok(Type::Range(Box::new(Type::Int))),

            BinOp::NullCoalesce => Ok(lt.clone()),
            BinOp::Pipeline     => Ok(rt.clone()),
            BinOp::Is           => Ok(Type::Bool),
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Two types are compatible if they're equal OR one is the "any" escape hatch.
    pub fn compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual { return true; }
        match (expected, actual) {
            (Type::Custom(a), _) if a == "any" => true,
            (_, Type::Custom(b)) if b == "any" => true,
            (Type::Custom(a), _) if self.env.is_generic(a) => true,
            (_, Type::Custom(b)) if self.env.is_generic(b) => true,
            (Type::List(a), Type::List(b))     => self.compatible(a, b),
            (Type::Option(a), Type::Option(b)) => self.compatible(a, b),
            (Type::Option(_), _)               => true, // permissive
            (Type::Future(a), Type::Future(b)) => self.compatible(a, b),
            (Type::Function(ap, ar), Type::Function(bp, br)) => {
                ap.len() == bp.len()
                    && ap.iter().zip(bp.iter()).all(|(a, b)| self.compatible(a, b))
                    && self.compatible(ar, br)
            }
            (Type::Custom(a), Type::Custom(b)) => {
                // resolve aliases
                let ra = self.env.resolve_alias(a);
                let rb = self.env.resolve_alias(b);
                match (ra, rb) {
                    (Some(ta), Some(tb)) => self.compatible(&ta, &tb),
                    (Some(ta), None)     => self.compatible(&ta, &Type::Custom(b.clone())),
                    (None, Some(tb))     => self.compatible(&Type::Custom(a.clone()), &tb),
                    (None, None)         => a == b,
                }
            }
            _ => false,
        }
    }

    fn check_type_valid(&self, ty: &Type) -> Result<()> {
        match ty {
            Type::Custom(name) => {
                // Only error on types that are neither generic params nor aliases
                // AND don't look like user-defined struct/enum names (we can't know at
                // this stage, so we skip undefined-type errors entirely — too many FPs)
                Ok(())
            }
            Type::List(inner) | Type::Option(inner) | Type::Future(inner)
            | Type::Range(inner) | Type::Reference(inner, _) | Type::Array(inner, _) => {
                self.check_type_valid(inner)
            }
            Type::Dict(k, v) => {
                self.check_type_valid(k)?;
                self.check_type_valid(v)
            }
            Type::Result(ok, err) => {
                self.check_type_valid(ok)?;
                self.check_type_valid(err)
            }
            Type::Tuple(ts) => ts.iter().try_for_each(|t| self.check_type_valid(t)),
            Type::Function(params, ret) => {
                params.iter().try_for_each(|t| self.check_type_valid(t))?;
                self.check_type_valid(ret)
            }
            Type::Generic(_, args) => args.iter().try_for_each(|t| self.check_type_valid(t)),
            _ => Ok(()),
        }
    }

    fn element_type(&self, ty: &Type) -> Type {
        match ty {
            Type::List(inner)  => *inner.clone(),
            Type::Range(_)     => Type::Int,
            Type::String       => Type::Char,
            Type::Tuple(ts)    => ts.first().cloned().unwrap_or(Type::Custom("any".into())),
            _                  => Type::Custom("any".into()),
        }
    }
}
