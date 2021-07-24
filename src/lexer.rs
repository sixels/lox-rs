use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(i64),

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Unknown,
}

#[derive(Clone, Debug)]
pub struct Lexer {
    pub buffer: String,
    position: usize,
}

impl Lexer {
    pub fn new(buffer: String) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn from_file<F: AsRef<Path>>(file: F) -> anyhow::Result<Self> {
        let mut source = File::open(file)?;
        let source_len = source.seek(SeekFrom::End(0))?;
        source.seek(SeekFrom::Start(0))?;

        let mut buffer = String::with_capacity((source_len) as usize + 1);

        // write to the buffer without reallocations
        source
            .take(source_len)
            .read_to_end(unsafe { buffer.as_mut_vec() })?;

        let scanner = Lexer::new(buffer);

        Ok(scanner)
    }

    fn tokenize_next(&mut self) -> anyhow::Result<Option<(TokenKind, usize)>> {
        let mut next_chars = self.buffer.chars().skip(self.position);
        if let (Some(current), next) = (next_chars.next(), next_chars.next()) {
            match current {
                // Single-character tokens
                '(' => Ok(Some((TokenKind::LeftParen, 1))),
                ')' => Ok(Some((TokenKind::RightParen, 1))),
                '{' => Ok(Some((TokenKind::LeftBrace, 1))),
                '}' => Ok(Some((TokenKind::RightBrace, 1))),
                ',' => Ok(Some((TokenKind::Comma, 1))),
                '.' => Ok(Some((TokenKind::Dot, 1))),
                '-' => Ok(Some((TokenKind::Minus, 1))),
                '+' => Ok(Some((TokenKind::Plus, 1))),
                ';' => Ok(Some((TokenKind::SemiColon, 1))),
                '/' => Ok(Some((TokenKind::Slash, 1))),
                '*' => Ok(Some((TokenKind::Star, 1))),

                // One or two character tokens
                '!' => {
                    if let Some('=') = next {
                        Ok(Some((TokenKind::BangEqual, 2)))
                    } else {
                        Ok(Some((TokenKind::Bang, 1)))
                    }
                }
                '=' => {
                    if let Some('=') = next {
                        Ok(Some((TokenKind::EqualEqual, 2)))
                    } else {
                        Ok(Some((TokenKind::Equal, 1)))
                    }
                }
                '>' => {
                    if let Some('=') = next {
                        Ok(Some((TokenKind::GreaterEqual, 2)))
                    } else {
                        Ok(Some((TokenKind::Greater, 1)))
                    }
                }
                '<' => {
                    if let Some('=') = next {
                        Ok(Some((TokenKind::LessEqual, 2)))
                    } else {
                        Ok(Some((TokenKind::Less, 1)))
                    }
                }

                // Literals
                '"' => self.tokenize_next_string(),
                ch @ '_' | ch if ch.is_alphabetic() => {
                    let (ident, length) = self.tokenize_next_identifier()?.unwrap();
                    if let TokenKind::Identifier(ident_str) = &ident {
                        // check if the identifier is a keyword
                        if let Some(keyword) = is_keyword(ident_str) {
                            Ok(Some((keyword, length)))
                        } else {
                            Ok(Some((ident, length)))
                        }
                    } else {
                        unreachable!()
                    }
                }
                ch if ch.is_digit(10) => self.tokenize_next_number(),

                // Unknwon character
                _ => Err(anyhow::Error::msg(format!(
                    "Unknown token: {}",
                    &self.buffer[self.position..self.position + 10]
                ))),
            }
        } else {
            // EOF
            Ok(None)
        }
    }

    fn take_all_next<F>(&self, matcher: F) -> (&str, usize)
    where
        F: Fn(char) -> bool,
    {
        let (str_taken, bytes_taken) = take_all(&self.buffer[self.position..], matcher);
        (str_taken, bytes_taken)
    }

    fn skip_whitespaces(&self) -> usize {
        let (_, skipped) = self.take_all_next(|ch| ch.is_whitespace());
        skipped
    }

    fn tokenize_next_identifier(&self) -> anyhow::Result<Option<(TokenKind, usize)>> {
        let (ident, length) = self.take_all_next(|ch| ch.is_alphanumeric());
        Ok(Some((TokenKind::Identifier(ident.to_string()), length)))
    }
    fn tokenize_next_string(&self) -> anyhow::Result<Option<(TokenKind, usize)>> {
        let (string, length) = take_all(&self.buffer[self.position + 1..], |ch| ch != '"');
        if self.buffer.chars().nth(self.position + length + 1) == Some('"') {
            Ok(Some((TokenKind::String(string.to_string()), length + 2)))
        } else {
            Err(anyhow::Error::msg("Unmatched string quotes"))
        }
    }
    fn tokenize_next_number(&self) -> anyhow::Result<Option<(TokenKind, usize)>> {
        let (number, length) = self.take_all_next(|ch| ch.is_digit(10));
        let number_parsed = number.parse()?;

        Ok(Some((TokenKind::Number(number_parsed), length)))
    }
}

impl Iterator for Lexer {
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        self.position += self.skip_whitespaces();

        if let Some((token, length)) = self.tokenize_next().unwrap().take() {
            self.position += length;
            Some(token)
        } else {
            None
        }
    }
}

fn take_all<'a, F>(data: &'a str, matcher: F) -> (&'a str, usize)
where
    F: Fn(char) -> bool,
{
    let mut index = 0;

    for ch in data.chars() {
        if !matcher(ch) {
            return (&data[..index], index);
        }

        index += ch.len_utf8();
    }

    (data, index)
}

fn is_keyword<'a>(data: &'a str) -> Option<TokenKind> {
    let keywords: HashMap<&'static str, TokenKind> = vec![
        ("and", TokenKind::And),
        ("class", TokenKind::Class),
        ("else", TokenKind::Else),
        ("false", TokenKind::False),
        ("fun", TokenKind::Fun),
        ("for", TokenKind::For),
        ("if", TokenKind::If),
        ("nil", TokenKind::Nil),
        ("or", TokenKind::Or),
        ("print", TokenKind::Print),
        ("return", TokenKind::Return),
        ("super", TokenKind::Super),
        ("this", TokenKind::This),
        ("true", TokenKind::True),
        ("var", TokenKind::Var),
        ("while", TokenKind::While),
    ]
    .into_iter()
    .collect();

    keywords.get(data).and_then(|token| Some(token.clone()))
}
