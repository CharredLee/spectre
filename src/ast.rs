#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Identifier(String),
    Integer(i64),
    Float(f64),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Term>,
    },
    FunctionCall {
        name: String,
        args: Vec<Term>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Box<Term>,
        right: Box<Term>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Term>,
    },
    SyntaxDefinition {
        name: String,
        pattern: String,
        precedence: usize,
        scope: Scope,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Scope {
    Global,
    Local,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Neg,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Times,
    Div,
    Pow,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Pow => 3,
            BinaryOperator::Times | BinaryOperator::Div => 2,
            BinaryOperator::Plus | BinaryOperator::Minus => 1,
        }
    }

    pub fn associativity(&self) -> Associativity {
        match self {
            BinaryOperator::Pow => Associativity::Right,
            _ => Associativity::Left,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Associativity {
    Left,
    Right,
}
