/// A whole file
#[derive(Clone, Debug, PartialEq)]
pub struct Grammar {
    pub productions: Vec<Production>,
}

/// One line such as
///     125 digits = digit { digit } .
#[derive(Clone, Debug, PartialEq)]
pub struct Production {
    /// Optional numeric label at the start of the line.
    pub index: Option<u32>,
    /// Left-hand side non-terminal
    pub lhs: Ident,
    /// Right-hand side tree
    pub rhs: Expr,
}

/// Anything on the right-hand side
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// a | b | c
    Choice(Vec<Expr>),

    /// a b c   (sequence)
    Sequence(Vec<Expr>),

    /// [ a ]   – zero-or-one
    Optional(Box<Expr>),

    /// { a }   – zero-or-more
    Repeat(Box<Expr>),

    /// (…)     – purely for grouping / precedence
    Group(Box<Expr>),

    /// leaf symbols
    Atom(Atom),
}

/// Terminals
#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    /// 'abs', '0', '--', '\xA', …
    Terminal(String),

    /// digit, width_spec, built_in_function, …
    NonTerminal(Ident),
}

/// Identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident(pub String);
