use crate::reader::JsonReader;
use crate::value::Number;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::iter::Peekable;

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

pub struct JsonTokenizer<T>
where
    T: Read + Seek,
{
    tokens: Vec<Token>,
    iterator: Peekable<JsonReader<T>>,
}

impl<T> JsonTokenizer<T>
where
    T: Read + Seek,
{
    pub fn new(reader: File) -> JsonTokenizer<File> {
        let json_reader = JsonReader::<File>::new(BufReader::new(reader));

        JsonTokenizer {
            iterator: json_reader.peekable(),
            tokens: vec![],
        }
    }

    pub fn from_bytes(input: &'static [u8]) -> JsonTokenizer<Cursor<&'static [u8]>> {
        let json_reader = JsonReader::<Cursor<&'static [u8]>>::from_bytes(input);

        JsonTokenizer {
            iterator: json_reader.peekable(),
            tokens: vec![],
        }
    }

    #[instrument]
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

    #[instrument]
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

    #[instrument]
    pub fn tokenize_json(&mut self) -> Result<&[Token], ()> {
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

        Ok(&self.tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::JsonParser;
    use crate::value::Number::F64;
    use std::collections::HashMap;
    use std::io::Cursor;

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

        let mut tokenizer = JsonTokenizer::<BufReader<Cursor<&[u8]>>>::from_bytes(input.as_bytes());
        let tokens = tokenizer.tokenize_json().unwrap();

        assert_eq!(expected_tokens.to_vec(), tokens);
    }

    #[test]
    fn parsed_json() {
        use crate::value::Value::*;

        let input = r#"{"pairs":[{"x0":95.26235434764715,"y0":-33.78221816487377,"x1":41.844453001935875,"y1":-78.10213222087448},{"x0":115.42029308864215,"y0":87.52060937339934,"x1":83.39640643072113,"y1":28.643090267505812},{"sample":"string sample","nullable":null}]}"#;
        let json_parser = JsonParser::parse_from_bytes(input.as_bytes());

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

        assert_eq!(json_parser.unwrap(), Object(pairs));
    }

    #[test]
    fn parse_utf8_json() {
        use crate::value::Value::*;

        let input = r#"{"key1":"ࠄࠀࠆࠄࠀࠁࠃ","key2":"value2"}"#;
        let json_parser = JsonParser::parse_from_bytes(input.as_bytes());

        let mut pairs = HashMap::new();
        pairs.insert("key1".to_string(), String("ࠄࠀࠆࠄࠀࠁࠃ".to_string()));
        pairs.insert("key2".to_string(), String("value2".to_string()));

        assert_eq!(json_parser.unwrap(), Object(pairs));
    }
}
