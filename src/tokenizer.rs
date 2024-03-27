//! Implementation of the tokenizer

use strum::EnumString;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Keyword {
    If, Fi,
    // TODO ...
}

// i am not storing whitespace tokens, this may bite me in the arse later?
#[derive(Debug)]
pub enum Token {
    /// Reserved keyword
    Keyword(Keyword),

    /// Identifier, [A-Za-z0-9_] basically
    Identifier(String),

    /// Integer, always signed
    Integer(i64),

    // /// Float, non posix compliant but eh
    // Float(f64),

    /// A regular string, holds type of string as well
    String(String, char),

    // /// Heredoc, like in shell <<EOF ... EOF
    // Heredoc(String),

    /// Parentheses, {} [] ()
    Paren(char),

    /// Any kind of operator, ==, ! ~ % ^ & > >>
    Symbol(String),

    /// Newline with line it was on
    Newline(usize),
}

/// Token type which stores position of the token and reference to the buffer
#[derive(Debug)]
pub struct TokenWithInfo {
    /// Index of start of the token
    pub start: usize,

    /// Index of end of the token
    pub end: usize,

    /// Reference to original buffer where it was parsed
    pub buffer: Rc<String>,

    /// The actual token
    pub token: Token,
}

pub fn tokenize(string: Rc<String>) -> Result<Vec<TokenWithInfo>, ()> {
    let mut tokens: Vec<TokenWithInfo> = vec![];
    let mut iter = string.chars().into_iter().enumerate().peekable();
    let mut line = 1;
    while let Some((i, ch)) = iter.next() {
        match ch {
            '\n' => {
                tokens.push(TokenWithInfo {
                    start: i,
                    end: i + 1,
                    buffer: string.clone(),
                    token: Token::Newline(line),
                });

                line += 1;
            },

            // TODO support unicode maybe?
            // start identifier if valid starting character
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut identifier = ch.to_string();

                // check next characters and build the identifier char by char
                while let Some((_, next)) = iter.peek() {
                    match next {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            identifier.push(iter.next().unwrap().1);
                        },
                        _ => break,
                    }
                }

                // test if it is a keyword otherwise save as an identifier
                match Keyword::from_str(identifier.as_str()) {
                    Ok(keyword) => tokens.push(TokenWithInfo {
                        start: i,
                        end: i + identifier.len(),
                        buffer: string.clone(),
                        token: Token::Keyword(keyword),
                    }),
                    Err(_) => tokens.push(TokenWithInfo {
                        start: i,
                        end: i + identifier.len(),
                        buffer: string.clone(),
                        token: Token::Identifier(identifier),
                    }),
                }
            },

            // TODO support negative numbers
            '0'..='9' => {
                let mut raw = ch.to_string();
                let mut integer: i64;

                match iter.peek() {
                    // hex
                    Some((_, 'x')) => {
                        raw.push(iter.next().unwrap().1);

                        while let Some((_, next)) = iter.peek() {
                            match next {
                                '0'..='9' | 'a'..='f' | 'A'..='F' => raw.push(iter.next().unwrap().1),
                                _ => break,
                            }
                        }

                        // it should not be possible to panic here
                        // NOTE: from_str_radix does not allow 0x prefix
                        integer = i64::from_str_radix(&raw[2..], 16).unwrap();
                    },

                    // TODO binary

                    // decimal
                    Some((_, '0'..='9')) => {
                        while let Some((_, next)) = iter.peek() {
                            match next {
                                '0'..='9' => raw.push(iter.next().unwrap().1),
                                _ => break,
                            }
                        }

                        integer = i64::from_str(&raw).unwrap();
                    },

                    // basically single digit decimal
                    _ => {
                        integer = i64::from_str(&raw).unwrap();
                    }
                }

                // save the token with its value
                tokens.push(TokenWithInfo {
                    start: i,
                    end: i + raw.len(),
                    buffer: string.clone(),
                    token: Token::Integer(integer)
                });
            },

            // parens
            '{' | '}' | '(' | ')' | '[' | ']' => {
                tokens.push(TokenWithInfo {
                    start: i,
                    end: i + 1,
                    buffer: string.clone(),
                    token: Token::Paren(ch)
                });
            },

            // string, including backtick
            '"' | '\'' | '`' => {
                let mut raw = ch.to_string();

                loop {
                    match iter.peek() {
                        // stop on newline TODO \ continouation
                        // Some((_, '\n')) => break,
                        // TODO should this be an error?

                        // stop only on the same kind of quote
                        Some((_, str_ch)) if *str_ch == ch => {
                            raw.push(iter.next().unwrap().1);
                            break;
                        },

                        // add other characters
                        Some((_, _)) => {
                            raw.push(iter.next().unwrap().1);
                        }

                        // eof so just stop
                        _ => break,
                    }
                }

                tokens.push(TokenWithInfo {
                    start: i,
                    end: i + raw.len(),
                    buffer: string.clone(),
                    // only double quote string can use variable substitution
                    token: Token::String(raw, ch),
                });
            }

            // all symbols in ascii
            '!'..='/' | ':'..='@' | '['..='`' | '{'..='~' => {
                let mut symbol = ch.to_string();

                // combine symbols like >> || && etc
                // TODO match <<<
                if let Some((_, next)) = iter.peek() {
                    match (ch.to_string() + &next.to_string()).as_str() {
                        ">>" | "<<" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "+=" | "-=" => {
                            symbol.push(iter.next().unwrap().1);
                        },
                        _ => {},
                    }
                }

                tokens.push(TokenWithInfo {
                    start: i,
                    end: i + symbol.len(),
                    buffer: string.clone(),
                    token: Token::Symbol(symbol)
                });
            },

            // ignore whitespace
            ' ' | '\t' => {},

            _ => {
                println!("Ignored '{}'", ch);
            },
        }
    }

    Ok(tokens)
}

