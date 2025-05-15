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

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Implementation(Implementation),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        type_: Option<Type>,
        initializer: Option<Expression>,
        is_mutable: bool,
    },
    Return(Option<Expression>),
    Expression(Expression),
    If {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    When {
        subject: Expression,
        arms: Vec<WhenArm>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    Assignment {
        target: String,
        value: Expression,
    },
}

#[derive(Debug, Clone)]
pub struct WhenArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    Constructor {
        name: String,
        fields: Vec<Pattern>,
    },
    Is(Type),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    Field {
        object: Box<Expression>,
        field: String,
    },
    Block(Block),
    If {
        condition: Box<Expression>,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    When {
        subject: Box<Expression>,
        arms: Vec<WhenArm>,
    },
    Await(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Type {
    Named {
        name: String,
        args: Vec<Type>,
    },
    Tuple(Vec<Type>),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
} 