use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Char,
    Void,
    Never,
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Future(Box<Type>),
    Range(Box<Type>),
    Custom(String),
    Generic(String, Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Reference(Box<Type>, bool), // (inner, is_mutable)
    Array(Box<Type>, Option<usize>),
    Trait(String),
    Union(Vec<Type>),
    SelfType,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int          => write!(f, "int"),
            Type::Float        => write!(f, "float"),
            Type::Bool         => write!(f, "bool"),
            Type::String       => write!(f, "string"),
            Type::Char         => write!(f, "char"),
            Type::Void         => write!(f, "void"),
            Type::Never        => write!(f, "!"),
            Type::SelfType     => write!(f, "Self"),
            Type::List(t)      => write!(f, "[{}]", t),
            Type::Option(t)    => write!(f, "{}?", t),
            Type::Future(t)    => write!(f, "Future<{}>", t),
            Type::Range(t)     => write!(f, "Range<{}>", t),
            Type::Trait(n)     => write!(f, "impl {}", n),
            Type::Custom(n)    => write!(f, "{}", n),
            Type::Reference(t, m) => {
                if *m { write!(f, "&mut {}", t) } else { write!(f, "&{}", t) }
            }
            Type::Array(t, Some(n)) => write!(f, "[{}; {}]", t, n),
            Type::Array(t, None)    => write!(f, "[{}]", t),
            Type::Dict(k, v)   => write!(f, "Dict<{}, {}>", k, v),
            Type::Tuple(ts)    => {
                let s: Vec<String> = ts.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", s.join(", "))
            }
            Type::Result(ok, err) => write!(f, "Result<{}, {}>", ok, err),
            Type::Generic(n, args) => {
                let s: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                write!(f, "{}<{}>", n, s.join(", "))
            }
            Type::Function(params, ret) => {
                let ps: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "fn({}) -> {}", ps.join(", "), ret)
            }
            Type::Union(ts)    => {
                let s: Vec<String> = ts.iter().map(|t| t.to_string()).collect();
                write!(f, "{}", s.join(" | "))
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Literals
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    None,
}

// ─────────────────────────────────────────────────────────────────────────────
// Operators
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    // Logical
    And, Or,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
    // Special
    NullCoalesce,   // ??
    Pipeline,       // |>
    Range,          // ..
    RangeInclusive, // ..=
    Is,             // is
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add   => "+",   BinOp::Sub => "-",
            BinOp::Mul   => "*",   BinOp::Div => "/",
            BinOp::Mod   => "%",   BinOp::Pow => "**",
            BinOp::Eq    => "==",  BinOp::Ne  => "!=",
            BinOp::Lt    => "<",   BinOp::Le  => "<=",
            BinOp::Gt    => ">",   BinOp::Ge  => ">=",
            BinOp::And   => "&&",  BinOp::Or  => "||",
            BinOp::BitAnd => "&",  BinOp::BitOr => "|",
            BinOp::BitXor => "^",  BinOp::Shl => "<<",
            BinOp::Shr   => ">>",
            BinOp::NullCoalesce    => "??",
            BinOp::Pipeline        => "|>",
            BinOp::Range           => "..",
            BinOp::RangeInclusive  => "..=",
            BinOp::Is              => "is",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // !
    BitNot, // ~
    Deref,  // *
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg    => write!(f, "-"),
            UnaryOp::Not    => write!(f, "!"),
            UnaryOp::BitNot => write!(f, "~"),
            UnaryOp::Deref  => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompoundOp {
    Add, Sub, Mul, Div, Mod, Pow, BitAnd, BitOr, BitXor,
}

impl fmt::Display for CompoundOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CompoundOp::Add => "+=", CompoundOp::Sub => "-=",
            CompoundOp::Mul => "*=", CompoundOp::Div => "/=",
            CompoundOp::Mod => "%=", CompoundOp::Pow => "**=",
            CompoundOp::BitAnd => "&=", CompoundOp::BitOr => "|=",
            CompoundOp::BitXor => "^=",
        };
        write!(f, "{}", s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Expressions
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),

    BinaryOp  { left: Box<Expr>, op: BinOp,   right: Box<Expr> },
    UnaryOp   { op: UnaryOp, expr: Box<Expr> },
    CompoundAssign { target: Box<Expr>, op: CompoundOp, value: Box<Expr> },

    /// Simple assignment — kept separate from BinaryOp::Eq
    Assign    { target: Box<Expr>, value: Box<Expr> },

    Call      { func: Box<Expr>, args: Vec<Expr> },
    MethodCall{ object: Box<Expr>, method: String, args: Vec<Expr> },
    Index     { target: Box<Expr>, index: Box<Expr> },
    FieldAccess { object: Box<Expr>, field: String },
    PathAccess  { path: Vec<String> },              // Foo::Bar::baz

    Lambda    { params: Vec<(String, Option<Type>)>, return_type: Option<Type>, body: Box<Expr> },
    Block     { stmts: Vec<Stmt>, trailing: Option<Box<Expr>> },

    If { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Option<Box<Expr>> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },

    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Tuple(Vec<Expr>),

    ListComprehension {
        expr:      Box<Expr>,
        var_name:  String,
        iterable:  Box<Expr>,
        condition: Option<Box<Expr>>,
    },

    Await  { expr: Box<Expr> },
    Try    { expr: Box<Expr> },

    TypeCast { expr: Box<Expr>, target_type: Type },

    StructInit { name: String, fields: Vec<(String, Expr)> },

    Return(Option<Box<Expr>>),
    Break(Option<Box<Expr>>),
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard:   Option<Box<Expr>>,
    pub body:    Box<Expr>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Patterns
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    Identifier(String),
    Binding { name: String, pattern: Box<Pattern> },
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>),
    Struct { name: String, fields: Vec<(String, Pattern)>, rest: bool },
    EnumVariant { path: Vec<String>, fields: Vec<Pattern> },
    Or(Vec<Pattern>),
    Range { start: Box<Expr>, end: Box<Expr>, inclusive: bool },
    Rest,
}

// ─────────────────────────────────────────────────────────────────────────────
// Statements
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),

    Let {
        name:      String,
        type_hint: Option<Type>,
        value:     Expr,
        mutable:   bool,
    },

    Function {
        name:           String,
        generic_params: Vec<String>,
        params:         Vec<Param>,
        return_type:    Type,
        body:           Vec<Stmt>,
        is_async:       bool,
        is_public:      bool,
    },

    Struct {
        name:           String,
        generic_params: Vec<String>,
        fields:         Vec<StructField>,
        methods:        Vec<Stmt>,
        is_public:      bool,
    },

    Enum {
        name:           String,
        generic_params: Vec<String>,
        variants:       Vec<EnumVariant>,
        methods:        Vec<Stmt>,
        is_public:      bool,
    },

    Trait {
        name:           String,
        generic_params: Vec<String>,
        super_traits:   Vec<String>,
        methods:        Vec<TraitMethod>,
        is_public:      bool,
    },

    Impl {
        generic_params: Vec<String>,
        trait_name:     Option<String>,
        type_name:      String,
        methods:        Vec<Stmt>,
    },

    Import {
        path:  String,
        items: Vec<ImportItem>,
    },

    Use {
        path:    String,
        as_name: Option<String>,
    },

    Const {
        name:      String,
        type_hint: Type,
        value:     Expr,
        is_public: bool,
    },

    TypeAlias {
        name:           String,
        generic_params: Vec<String>,
        alias_type:     Type,
        is_public:      bool,
    },

    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,

    While { condition: Expr, body: Vec<Stmt>, label: Option<String> },
    Loop  { body: Vec<Stmt>, label: Option<String> },
    For   { var: String, iterator: Expr, body: Vec<Stmt>, label: Option<String> },

    If {
        condition:  Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },

    Match {
        expr: Expr,
        arms: Vec<(Pattern, Option<Expr>, Vec<Stmt>)>,
    },

    Try {
        block:         Vec<Stmt>,
        catch_blocks:  Vec<CatchBlock>,
        finally_block: Option<Vec<Stmt>>,
    },

    Async { block: Vec<Stmt> },
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper types used inside Stmt/Expr nodes
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name:    String,
    pub ty:      Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name:      String,
    pub ty:        Type,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name:   String,
    pub fields: EnumVariantFields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantFields {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name:        String,
    pub params:      Vec<Param>,
    pub return_type: Type,
    pub body:        Option<Vec<Stmt>>,
    pub is_async:    bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportItem {
    Name(String),
    Alias(String, String), // original, alias
    All,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchBlock {
    pub binding:    Option<String>,
    pub error_type: Option<Type>,
    pub body:       Vec<Stmt>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Program root
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Program {
    pub statements:  Vec<Stmt>,
    pub source_file: Option<String>,
}

impl Program {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self { statements, source_file: None }
    }

    pub fn with_source(statements: Vec<Stmt>, source_file: String) -> Self {
        Self { statements, source_file: Some(source_file) }
    }
}
