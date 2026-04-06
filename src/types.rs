use std::borrow::Cow;

pub type Value = i64;
pub type ClacStack = Vec<Value>;

type FunctionIndex = usize;

pub type FuncName<'a> = Cow<'a, str>;

#[derive(Debug, Clone)]
pub enum FunctionRef<'a> {
    Resolved(FunctionIndex),
    Unresolved(FuncName<'a>),
}

#[derive(Debug, Clone)]
pub enum Token<'func_str> {
    // data
    Literal(Value),
    FunctionCall(FunctionRef<'func_str>),

    // side effects
    Quit,
    Print,

    // syscall
    // Ptr,
    // Syscall,

    // stack manipulation
    Drop,
    Swap,
    Rot,

    If,
    Pick,
    Skip,

    // function stuff
    Colon,
    Semicolon,
}

pub type Code<'func_str> = Vec<Token<'func_str>>;

#[derive(Debug)]
pub enum Function<'func_str> {
    Clac(Code<'func_str>),

    Native(fn(&mut ClacStack)),

    ClacOp(fn(Value, Value) -> Value),
}

// pub type FuncMap = ahash::AHashMap<String, FunctionIndex>;
pub type CallStack<'a> = Vec<&'a [Token<'a>]>;

#[derive(Debug)]
pub struct FuncMap<'func_str> {
    // TODO: this doesn't need to be owned
    pub map: ahash::AHashMap<FuncName<'func_str>, FunctionIndex>,
    pub functions: Vec<Function<'func_str>>,
}

#[derive(Debug)]
/// The primary struct representing the state of the Clac++ machine.
pub struct ClacState<'func_str> {
    pub stack: ClacStack,
    pub funcmap: FuncMap<'func_str>,
}

pub enum ExecRes<'a> {
    Executed,
    Skip(usize),
    RecursiveCall(&'a [Token<'a>]),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ExecError {
    UnknownFunction(String),
    MissingArguments,
    InvalidSkip,
    InvalidPick,
    BadFunctionDefinition,
    Quit,
}
