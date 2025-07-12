use lalrpop_util::lalrpop_mod;
pub mod ast;
pub mod lexer;
lalrpop_mod!(grammar);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_choice() -> anyhow::Result<()> {
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
        Ok(())
    }

    #[test]
    fn test_parse_sequence() -> anyhow::Result<()> {
        let source = "25 assignment = variable '=' expression .";
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        let ast = parser.parse(lexer)?;
        let expected = ast::Grammar {
            productions: vec![ast::Production {
                index: Some(25),
                lhs: ast::Ident("assignment".into()),
                rhs: ast::Expr::Sequence(vec![
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("variable".into()))),
                    ast::Expr::Atom(ast::Atom::Terminal("=".into())),
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("expression".into()))),
                ]),
            }],
        };
        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn test_parse_optional() -> anyhow::Result<()> {
        let source = "30 function_call = function_name [ argument_list ] .";
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        let ast = parser.parse(lexer)?;
        let expected = ast::Grammar {
            productions: vec![ast::Production {
                index: Some(30),
                lhs: ast::Ident("function_call".into()),
                rhs: ast::Expr::Sequence(vec![
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("function_name".into()))),
                    ast::Expr::Optional(Box::new(ast::Expr::Atom(ast::Atom::NonTerminal(
                        ast::Ident("argument_list".into()),
                    )))),
                ]),
            }],
        };
        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn test_parse_repeat() -> anyhow::Result<()> {
        let source = "125 digits = digit { digit } .";
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        let ast = parser.parse(lexer)?;
        let expected = ast::Grammar {
            productions: vec![ast::Production {
                index: Some(125),
                lhs: ast::Ident("digits".into()),
                rhs: ast::Expr::Sequence(vec![
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("digit".into()))),
                    ast::Expr::Repeat(Box::new(ast::Expr::Atom(ast::Atom::NonTerminal(
                        ast::Ident("digit".into()),
                    )))),
                ]),
            }],
        };
        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn test_parse_group() -> anyhow::Result<()> {
        let source = "40 expression = term ( '+' | '-' ) term .";
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        let ast = parser.parse(lexer)?;
        let expected = ast::Grammar {
            productions: vec![ast::Production {
                index: Some(40),
                lhs: ast::Ident("expression".into()),
                rhs: ast::Expr::Sequence(vec![
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("term".into()))),
                    ast::Expr::Group(Box::new(ast::Expr::Choice(vec![
                        ast::Expr::Atom(ast::Atom::Terminal("+".into())),
                        ast::Expr::Atom(ast::Atom::Terminal("-".into())),
                    ]))),
                    ast::Expr::Atom(ast::Atom::NonTerminal(ast::Ident("term".into()))),
                ]),
            }],
        };
        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn test_parse_iso_10303_11_2004_bnf() -> anyhow::Result<()> {
        let source = std::fs::read_to_string("../cadk-express/data/iso-10303-11-2004.bnf")?;
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        let ast = parser.parse(lexer)?;
        
        // Basic sanity checks
        assert!(!ast.productions.is_empty(), "BNF file should contain productions");
        
        // Check that we have some expected keywords
        let has_abs = ast.productions.iter().any(|p| p.lhs.0 == "ABS");
        let has_entity = ast.productions.iter().any(|p| p.lhs.0 == "ENTITY");
        let has_schema = ast.productions.iter().any(|p| p.lhs.0 == "SCHEMA");
        
        assert!(has_abs, "Should have ABS production");
        assert!(has_entity, "Should have ENTITY production");
        assert!(has_schema, "Should have SCHEMA production");
        
        Ok(())
    }

    #[test]
    fn test_parse_iso_10303_21_2002_bnf() -> anyhow::Result<()> {
        let source = std::fs::read_to_string("../cadk-step/data/iso-10303-21-2002.bnf")?;
        let lexer = lexer::Lexer::new(&source);
        let parser = grammar::GrammarParser::new();
        
        match parser.parse(lexer) {
            Ok(ast) => {
                // Basic sanity checks
                assert!(!ast.productions.is_empty(), "BNF file should contain productions");
                
                // Check that we have some expected STEP file format productions
                let has_exchange_file = ast.productions.iter().any(|p| p.lhs.0 == "exchange_file");
                let has_header_section = ast.productions.iter().any(|p| p.lhs.0 == "header_section");
                let has_data_section = ast.productions.iter().any(|p| p.lhs.0 == "data_section");
                
                assert!(has_exchange_file, "Should have exchange_file production");
                assert!(has_header_section, "Should have header_section production");
                assert!(has_data_section, "Should have data_section production");
                
                Ok(())
            }
            Err(e) => {
                // Extract location information from the error
                let error_msg = format!("{:?}", e);
                
                // Try to find the location in the source where the error occurred
                if let Some(location) = extract_error_location(&error_msg) {
                    let lines: Vec<&str> = source.lines().collect();
                    let line_start = source[..location].matches('\n').count();
                    let context_start = line_start.saturating_sub(2);
                    let context_end = (line_start + 3).min(lines.len());
                    
                    eprintln!("Parse error at position {}", location);
                    eprintln!("Context around error:");
                    for (i, line) in lines[context_start..context_end].iter().enumerate() {
                        let line_num = context_start + i + 1;
                        eprintln!("{:4}: {}", line_num, line);
                    }
                    
                    // Show the specific character/token that failed
                    if location < source.len() {
                        let error_char = &source[location..location.saturating_add(20)];
                        eprintln!("Failed at: '{}'", error_char);
                    }
                }
                
                Err(anyhow::anyhow!("Failed to parse ISO 10303-21-2002 BNF: {}", error_msg))
            }
        }
    }
    
    fn extract_error_location(error_msg: &str) -> Option<usize> {
        // Try to extract location from error message
        // This is a simple heuristic that might need adjustment based on actual error format
        if let Some(pos) = error_msg.find("location: ") {
            let location_str = &error_msg[pos + 10..];
            if let Some(end) = location_str.find(|c: char| !c.is_numeric()) {
                location_str[..end].parse().ok()
            } else {
                location_str.parse().ok()
            }
        } else if let Some(pos) = error_msg.find(" at ") {
            let location_str = &error_msg[pos + 4..];
            if let Some(end) = location_str.find(|c: char| !c.is_numeric()) {
                location_str[..end].parse().ok()
            } else {
                location_str.parse().ok()
            }
        } else {
            None
        }
    }
}
