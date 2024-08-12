use crate::types::{MalCollection, MalType};
use anyhow::{anyhow, Ok, Result};
use regex::Regex;

#[derive(Debug)]
pub enum MalToken {
    Whitespace(String),
    Special(String),
    Quote(String),
    CharSeq(String),
    Symbols(String),
}

#[derive(Debug)]
pub struct Lexer {
    position: usize,
    tokens: Vec<String>,
}

impl Lexer {
    pub fn tokenize(buffer: &str) -> Self {
        let regex = regex::Regex::new(
            r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###,
        )
        .unwrap();

        let mut tokens = vec![];

        for capture in regex.captures_iter(buffer) {
            if let Some(token) = capture.get(1) {
                match token.as_str() {
                    tok if tok.len() == 0 || tok.starts_with(';') => (),
                    tok => tokens.push(tok.to_string()),
                }
            }
        }

        Self {
            position: 0,
            tokens,
        }
    }

    pub fn next(&mut self) -> Option<&String> {
        self.position += 1;
        self.tokens.get(self.position - 1)
    }

    pub fn peek(&self) -> Option<&String> {
        self.tokens.get(self.position)
    }
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<Vec<MalType>> {
        let mut types = vec![];

        while let Some(val) = self.read_next()? {
            types.push(val)
        }

        Ok(types)
    }

    fn read_next(&mut self) -> Result<Option<MalType>> {
        let val = if let Some(val) = self.lexer.peek() {
            val
        } else {
            return Ok(None);
        };

        match val.as_str() {
            "" => Ok(None),

            "(" | "[" | "{" => self.read_collection(),
            ")" | "]" | "}" => anyhow::bail!("Received collection end while trying to read next"),
            _ => self.read_symbol(),
        }
    }

    fn read_collection(&mut self) -> Result<Option<MalType>> {
        // eat start
        let collection_type = MalCollection::get(self.lexer.next().unwrap().as_str());
        let mut list = vec![];

        // Take while next token is not END OF LIST
        loop {
            let token = if let Some(next) = self.lexer.peek() {
                next
            } else {
                anyhow::bail!("Received EOF without ending collection")
            };

            if token.as_str() == collection_type.end() {
                // Eat end of list
                self.lexer.next();
                return Ok(Some(collection_type.into(list)));
            } else {
                if let Some(token) = self.read_next()? {
                    list.push(token)
                } else {
                    anyhow::bail!("Got None in read_next while reading list! {:?}", self)
                }
            }
        }
    }

    fn read_symbol(&mut self) -> Result<Option<MalType>> {
        let symbol = match self.lexer.next() {
            Some(symbol) => symbol,
            wat => anyhow::bail!("Unexpected token in read_symbol. {:?}", wat),
        };

        match symbol.as_str() {
            "true" => Ok(Some(MalType::Bool(true))),
            "false" => Ok(Some(MalType::Bool(false))),
            "nil" => Ok(Some(MalType::Nil)),
            number if number.chars().all(|x| x.is_ascii_digit()) => {
                let number: i64 = number.parse()?;
                Ok(Some(MalType::Number(number)))
            }
            number
                if number.starts_with('-')
                    && number.len() > 1
                    && number.chars().skip(1).all(|x| x.is_ascii_digit()) =>
            {
                let number: i64 = number.parse()?;
                Ok(Some(MalType::Number(number)))
            }
            str if str.starts_with('"') => {
                if str.len() == 1 {
                    anyhow::bail!("EOF: Received only opening quote")
                }
                if !str.ends_with('"') {
                    anyhow::bail!("EOF: String ended unexpectantly")
                }
                Ok(Some(MalType::String(str.to_string())))
            }
            other => Ok(Some(MalType::Symbol(other.to_string()))),
            // _ => anyhow::bail!("Received unexpected symbol. {:?}", symbol),
        }
    }
}
