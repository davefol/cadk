use logos::{Logos, SpannedIter};

#[derive(Debug, Default, Clone, PartialEq, thiserror::Error)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

impl std::fmt::Display for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn unquote(s: &str) -> &str {
    s.strip_prefix("'").unwrap().strip_suffix("'").unwrap()
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r";.*\n?", error = LexicalError)]
pub enum Token {
    #[token("=")]
    Equal,
    #[token(".")]
    Period,
    #[token("|")]
    Pipe,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    LineNumber(u32),
    #[regex("'[^']*'", |lex| unquote(lex.slice()).to_owned())]
    String(String),
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex(r"\\[A-Za-z](?:[0-9A-Fa-f]{1,2})?", |lex| lex.slice().to_owned())]
    EscapeSequence(String),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexer() -> anyhow::Result<()> {
        let mut lexer = Lexer::new("==");
        let spanned_token = lexer.next().unwrap()?;
        assert_eq!(spanned_token, (0, Token::Equal, 1));
        let spanned_token = lexer.next().unwrap()?;
        assert_eq!(spanned_token, (1, Token::Equal, 2));
        let spanned_token = lexer.next();
        assert_eq!(spanned_token, None);
        Ok(())
    }

    #[test]
    fn test_lex_bnf_rules() -> anyhow::Result<()> {
        let source = r"
         0 ABS = 'abs' .
         123 bit = '0' | '1' .
         125 digits = digit { digit } .
         219 function_call = ( built_in_function | function_ref ) [ actual_parameter_list ] .
         341 width_spec = '(' width ')' [ FIXED ] .
         149 tail_remark = '--' [ remark_tag ] { \a | \s | \x8 | \x9 | \xA | \xB | \xC | \xD } \n .
        ";
        let lexer = Lexer::new(source);
        let expected_tokens = [
            // 0 ABS = 'abs' .
            Token::LineNumber(0),
            Token::Identifier("ABS".into()),
            Token::Equal,
            Token::String("abs".into()),
            Token::Period,
            // 123 bit = '0' | '1' .
            Token::LineNumber(123),
            Token::Identifier("bit".into()),
            Token::Equal,
            Token::String("0".into()),
            Token::Pipe,
            Token::String("1".into()),
            Token::Period,
            // 125 digits = digit { digit } .
            Token::LineNumber(125),
            Token::Identifier("digits".into()),
            Token::Equal,
            Token::Identifier("digit".into()),
            Token::LeftBrace,
            Token::Identifier("digit".into()),
            Token::RightBrace,
            Token::Period,
            // 219 function_call = ( built_in_function | function_ref ) [ actual_parameter_list ] .
            Token::LineNumber(219),
            Token::Identifier("function_call".into()),
            Token::Equal,
            Token::LeftParen,
            Token::Identifier("built_in_function".into()),
            Token::Pipe,
            Token::Identifier("function_ref".into()),
            Token::RightParen,
            Token::LeftBracket,
            Token::Identifier("actual_parameter_list".into()),
            Token::RightBracket,
            Token::Period,
            // 341 width_spec = '(' width ')' [ FIXED ] .
            Token::LineNumber(341),
            Token::Identifier("width_spec".into()),
            Token::Equal,
            Token::String("(".into()),
            Token::Identifier("width".into()),
            Token::String(")".into()),
            Token::LeftBracket,
            Token::Identifier("FIXED".into()),
            Token::RightBracket,
            Token::Period,
            // 149 tail_remark = '--' [ remark_tag ] { \a | \s | \x8 | \x9 | \xA | \xB | \xC | \xD } \n .
            Token::LineNumber(149),
            Token::Identifier("tail_remark".into()),
            Token::Equal,
            Token::String("--".into()),
            Token::LeftBracket,
            Token::Identifier("remark_tag".into()),
            Token::RightBracket,
            Token::LeftBrace,
            Token::EscapeSequence(r"\a".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\s".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\x8".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\x9".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\xA".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\xB".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\xC".into()),
            Token::Pipe,
            Token::EscapeSequence(r"\xD".into()),
            Token::RightBrace,
            Token::EscapeSequence(r"\n".into()),
            Token::Period,
        ];
        let tokens: Vec<(usize, Token, usize)> = lexer.into_iter().map(|t| t.unwrap()).collect();
        for (actual, expected) in tokens.iter().zip(expected_tokens) {
            assert_eq!(
                actual.1,
                expected,
                "{}..{} {}",
                actual.0,
                actual.2,
                &source[(actual.0 - 10).max(0)..actual.2]
            );
        }
        Ok(())
    }
}
