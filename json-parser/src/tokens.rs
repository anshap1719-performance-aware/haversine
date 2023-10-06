use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufReader, Read};
use std::iter::Peekable;
use std::slice::Iter;
use std::str::from_utf8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    CurlyOpen,
    CurlyClose,
    Quotes,
    Colon,
    String(String),
    Number(Number),
    ArrayOpen,
    ArrayClose,
    Comma,
    Boolean(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(Number),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

struct JsonReader {
    reader: Box<BufReader<dyn Read>>,
    character_buffer: VecDeque<char>,
}

impl JsonReader {
    fn new(reader: BufReader<File>) -> Self {
        JsonReader {
            reader: Box::new(reader),
            character_buffer: VecDeque::with_capacity(4),
        }
    }

    fn from_string(reader: BufReader<&'static [u8]>) -> Self {
        JsonReader {
            reader: Box::new(reader),
            character_buffer: VecDeque::with_capacity(4),
        }
    }
}

impl Iterator for JsonReader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.character_buffer.is_empty() {
            return self.character_buffer.pop_front();
        }

        let mut utf8_buffer = [0, 0, 0, 0];
        let _ = self.reader.read(&mut utf8_buffer);

        if let Ok(string) = from_utf8(&utf8_buffer) {
            self.character_buffer = string.chars().collect();
            self.character_buffer.pop_front()
        } else {
            None
        }
    }
}

pub struct JsonParser {
    tokens: Vec<Token>,
    iterator: Peekable<JsonReader>,
}

impl JsonParser {
    pub fn new(reader: File) -> Self {
        let json_reader = JsonReader::new(BufReader::new(reader));

        Self {
            iterator: json_reader.peekable(),
            tokens: vec![],
        }
    }

    fn from_string(input: &'static str) -> Self {
        let json_reader = JsonReader::from_string(BufReader::new(input.as_bytes()));

        Self {
            iterator: json_reader.peekable(),
            tokens: vec![],
        }
    }

    fn parse_string(&mut self) -> Result<String, ()> {
        let mut string_characters = Vec::<char>::new();

        for character in self.iterator.by_ref() {
            if character == '"' {
                break;
            }

            string_characters.push(character);
        }

        Ok(String::from_iter(string_characters))
    }

    fn parse_number(&mut self) -> Result<Number, ()> {
        let mut number_characters = Vec::<char>::new();
        let mut is_decimal = false;

        while let Some(character) = self.iterator.peek() {
            match character {
                '-' => {
                    number_characters.push('-');
                    let _ = self.iterator.next();
                }
                digit @ '0'..='9' => {
                    number_characters.push(*digit);
                    let _ = self.iterator.next();
                }
                '.' => {
                    number_characters.push('.');
                    is_decimal = true;
                    let _ = self.iterator.next();
                }
                '}' | ',' | ']' | ':' => {
                    break;
                }
                _ => {
                    panic!("Unexpected character while parsing number: {character}")
                }
            }
        }

        if is_decimal {
            Ok(Number::F64(
                String::from_iter(number_characters)
                    .parse::<f64>()
                    .map_err(|_| ())?,
            ))
        } else {
            Ok(Number::I64(
                String::from_iter(number_characters)
                    .parse::<i64>()
                    .map_err(|_| ())?,
            ))
        }
    }

    fn parse_object(&self, iterator: &mut Peekable<Iter<Token>>) -> HashMap<String, Value> {
        let mut is_key = true;
        let mut current_key: Option<&str> = None;
        let mut value = HashMap::<String, Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => {
                    if let Some(current_key) = current_key {
                        value.insert(
                            current_key.to_string(),
                            Value::Object(self.parse_object(iterator)),
                        );
                    }
                }
                Token::CurlyClose => {
                    break;
                }
                Token::Quotes => {}
                Token::Colon => {
                    is_key = false;
                }
                Token::String(string) => {
                    if is_key {
                        current_key = Some(string);
                    } else if let Some(key) = current_key {
                        value.insert(key.to_string(), Value::String(string.clone()));
                        current_key = None;
                    }
                }
                Token::Number(number) => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), Value::Number(*number));
                        current_key = None;
                    }
                }
                Token::ArrayOpen => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), Value::Array(self.parse_array(iterator)));
                        current_key = None;
                    }
                }
                Token::ArrayClose => {}
                Token::Comma => is_key = true,
                Token::Boolean(boolean) => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), Value::Boolean(*boolean));
                        current_key = None;
                    }
                }
                Token::Null => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), Value::Null);
                        current_key = None;
                    }
                }
            }
        }

        value
    }

    fn parse_array(&self, iterator: &mut Peekable<Iter<Token>>) -> Vec<Value> {
        let mut internal_value = Vec::<Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => internal_value.push(Value::Object(self.parse_object(iterator))),
                Token::CurlyClose => {}
                Token::Quotes => {}
                Token::Colon => {}
                Token::String(string) => internal_value.push(Value::String(string.clone())),
                Token::Number(number) => internal_value.push(Value::Number(*number)),
                Token::ArrayOpen => internal_value.push(Value::Array(self.parse_array(iterator))),
                Token::ArrayClose => {
                    break;
                }
                Token::Comma => {}
                Token::Boolean(boolean) => internal_value.push(Value::Boolean(*boolean)),
                Token::Null => internal_value.push(Value::Null),
            }
        }

        internal_value
    }

    fn tokens_to_value(&self, tokens: &[Token]) -> Value {
        let mut iterator = tokens.iter().peekable();

        let mut value = Value::Null;

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => {
                    value = Value::Object(self.parse_object(&mut iterator));
                }
                Token::CurlyClose => {}
                Token::Quotes => {}
                Token::Colon => {}
                Token::String(string) => {
                    value = Value::String(string.clone());
                }
                Token::Number(number) => {
                    value = Value::Number(*number);
                }
                Token::ArrayOpen => {
                    value = Value::Array(self.parse_array(&mut iterator));
                }
                Token::ArrayClose => {}
                Token::Comma => {}
                Token::Boolean(boolean) => value = Value::Boolean(*boolean),
                Token::Null => value = Value::Null,
            }
        }

        value
    }

    pub fn parse_json(&mut self) -> Result<Value, ()> {
        while let Some(character) = self.iterator.peek() {
            match *character {
                '"' => {
                    self.tokens.push(Token::Quotes);

                    // Skip quote token since we already added it to the tokens list.
                    let _ = self.iterator.next();

                    let string = self.parse_string()?;

                    self.tokens.push(Token::String(string));
                    self.tokens.push(Token::Quotes);
                }
                '{' => {
                    self.tokens.push(Token::CurlyOpen);
                    let _ = self.iterator.next();
                }
                '}' => {
                    self.tokens.push(Token::CurlyClose);
                    let _ = self.iterator.next();
                }
                '[' => {
                    self.tokens.push(Token::ArrayOpen);
                    let _ = self.iterator.next();
                }
                ']' => {
                    self.tokens.push(Token::ArrayClose);
                    let _ = self.iterator.next();
                }
                ',' => {
                    self.tokens.push(Token::Comma);
                    let _ = self.iterator.next();
                }
                ':' => {
                    self.tokens.push(Token::Colon);
                    let _ = self.iterator.next();
                }
                '-' | '0'..='9' => {
                    let number = self.parse_number()?;
                    self.tokens.push(Token::Number(number));
                }
                '\0' => break,
                'n' => {
                    self.tokens.push(Token::Null);

                    // Advance iterator by 4 for null character
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                }
                't' => {
                    self.tokens.push(Token::Boolean(true));

                    // Advance iterator by 4 for true keyword
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                }
                'f' => {
                    self.tokens.push(Token::Boolean(false));

                    // Advance iterator by 5 for false character
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                    let _ = self.iterator.next();
                }
                character => {
                    if character.is_ascii_whitespace() {
                        continue;
                    }

                    panic!("Unexpected character: ;{character};")
                }
            }
        }

        Ok(self.tokens_to_value(&self.tokens))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tokens::Number::F64;

    #[test]
    fn test_tokenizer() {
        use Token::*;

        let input = r#"{"pairs":[{"x0":95.26235434764715,"y0":-33.78221816487377,"x1":41.844453001935875,"y1":-78.10213222087448},{"x0":115.42029308864215,"y0":87.52060937339934,"x1":83.39640643072113,"y1":28.643090267505812}]}"#;

        let expected_tokens = [
            CurlyOpen,
            Quotes,
            String("pairs".to_string()),
            Quotes,
            Colon,
            ArrayOpen,
            CurlyOpen,
            Quotes,
            String("x0".to_string()),
            Quotes,
            Colon,
            Number(F64(95.26235434764715)),
            Comma,
            Quotes,
            String("y0".to_string()),
            Quotes,
            Colon,
            Number(F64(-33.78221816487377)),
            Comma,
            Quotes,
            String("x1".to_string()),
            Quotes,
            Colon,
            Number(F64(41.844453001935875)),
            Comma,
            Quotes,
            String("y1".to_string()),
            Quotes,
            Colon,
            Number(F64(-78.10213222087448)),
            CurlyClose,
            Comma,
            CurlyOpen,
            Quotes,
            String("x0".to_string()),
            Quotes,
            Colon,
            Number(F64(115.42029308864215)),
            Comma,
            Quotes,
            String("y0".to_string()),
            Quotes,
            Colon,
            Number(F64(87.52060937339934)),
            Comma,
            Quotes,
            String("x1".to_string()),
            Quotes,
            Colon,
            Number(F64(83.39640643072113)),
            Comma,
            Quotes,
            String("y1".to_string()),
            Quotes,
            Colon,
            Number(F64(28.643090267505812)),
            CurlyClose,
            ArrayClose,
            CurlyClose,
        ];

        let mut json_parser = JsonParser::from_string(input);

        assert!(json_parser.parse_json().is_ok());
        assert_eq!(expected_tokens.to_vec(), json_parser.tokens);
    }

    #[test]
    fn parsed_json() {
        use Value::*;

        let input = r#"{"pairs":[{"x0":95.26235434764715,"y0":-33.78221816487377,"x1":41.844453001935875,"y1":-78.10213222087448},{"x0":115.42029308864215,"y0":87.52060937339934,"x1":83.39640643072113,"y1":28.643090267505812},{"sample":"string sample","nullable":null}]}"#;
        let mut json_parser = JsonParser::from_string(input);

        let mut entry1 = HashMap::new();
        entry1.insert("y0".to_string(), Number(F64(-33.78221816487377)));
        entry1.insert("x0".to_string(), Number(F64(95.26235434764715)));
        entry1.insert("y1".to_string(), Number(F64(-78.10213222087448)));
        entry1.insert("x1".to_string(), Number(F64(41.844453001935875)));

        let mut entry2 = HashMap::new();
        entry2.insert("y0".to_string(), Number(F64(87.52060937339934)));
        entry2.insert("x0".to_string(), Number(F64(115.42029308864215)));
        entry2.insert("x1".to_string(), Number(F64(83.39640643072113)));
        entry2.insert("y1".to_string(), Number(F64(28.643090267505812)));

        let mut entry3 = HashMap::new();
        entry3.insert("sample".to_string(), String("string sample".to_string()));
        entry3.insert("nullable".to_string(), Null);

        let mut pairs = HashMap::new();
        pairs.insert(
            "pairs".to_string(),
            Array(vec![Object(entry1), Object(entry2), Object(entry3)]),
        );

        assert_eq!(json_parser.parse_json().unwrap(), Object(pairs));
    }
}
