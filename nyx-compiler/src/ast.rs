use std::fmt;

#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub name: String,
    pub bounds: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub supertraits: Vec<Type>,
    pub items: Vec<TraitItem>,
}

#[derive(Debug, Clone)]
pub enum TraitItem {
    Method(TraitMethod),
    Type(TraitType),
    Const(TraitConst),
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Option<Block>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct TraitType {
    pub name: String,
    pub bounds: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct TraitConst {
    pub name: String,
    pub type_: Type,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Implementation {
    pub type_params: Vec<TypeParameter>,
    pub target_type: Type,
    pub trait_name: Option<Type>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// A module-aware program that can handle both standalone functions and modules
#[derive(Debug, Clone)]
pub struct ModuleAwareProgram {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub modules: Vec<ModuleDecl>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Implementation(Implementation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        mutable: bool,
        type_: Option<Type>,
        initializer: Option<Expression>,
    },
    Assignment {
        name: String,
        value: Expression,
    },
    Expression(Expression),
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        condition: Expression,
        body: Block,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::Bool => write!(f, "Bool"),
            Type::String => write!(f, "String"),
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Or => write!(f, "||"),
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Not => write!(f, "!"),
        }
    }
}

/// A module path for referencing items in other modules
#[derive(Debug, Clone, PartialEq)]
pub struct ModulePath {
    pub segments: Vec<String>,
}

impl ModulePath {
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }
    
    pub fn single(name: String) -> Self {
        Self { segments: vec![name] }
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segments.join("."))
    }
}

/// Visibility modifier for items
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

/// An import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: ModulePath,
    pub items: Option<Vec<String>>, // None means import all, Some means import specific items
    pub alias: Option<String>,
}

/// A module declaration
#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub name: String,
    pub visibility: Visibility,
    pub items: Vec<Item>,
} 