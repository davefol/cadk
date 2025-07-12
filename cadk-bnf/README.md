# CadK BNF
Parser for .bbf files

Used to translate iso-10303-11-2004.bnf and iso-10303-21-2004.bnf files into
.lalrpop format for building a STEP file parser.

## Usage
```{rust}
let source = "12 a = 'b' | 'c' .";
let lexer = lexer::Lexer::new(&source);
let parser = grammar::GrammarParser::new();
let ast = parser.parse(lexer)?;
let expected = ast::Grammar {
    productions: vec![ast::Production {
        index: Some(12),
        lhs: ast::Ident("a".into()),
        rhs: ast::Expr::Choice(vec![
            ast::Expr::Atom(ast::Atom::Terminal("b".into())),
            ast::Expr::Atom(ast::Atom::Terminal("c".into())),
        ]),
    }],
};
assert_eq!(ast, expected);
```