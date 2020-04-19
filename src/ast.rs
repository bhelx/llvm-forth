#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Ast {
    Push(i64),
    Word(String),
    Variable(String),
    DotQuote(String),
    Phrase(Vec<Ast>),
    Conditional {
        consequent: Box<Ast>,
        alternative: Option<Box<Ast>>,
    },
    DoLoop(Box<Ast>),
    Definition(Box<Ast>),
}
