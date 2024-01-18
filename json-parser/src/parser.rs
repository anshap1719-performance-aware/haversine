use crate::tokens::{JsonTokenizer, Token};
use crate::value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::iter::Peekable;
use std::slice::Iter;

/// Main parser which is the entrypoint for parsing JSON.
pub struct JsonParser;

impl JsonParser {
    /// Parse an object from token stream.
    fn process_object(iterator: &mut Peekable<Iter<Token>>) -> HashMap<String, Value> {
        let mut is_key = true;
        let mut current_key: Option<&str> = None;
        let mut value = HashMap::<String, Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => {
                    if let Some(current_key) = current_key {
                        value.insert(
                            current_key.to_string(),
                            Value::Object(Self::process_object(iterator)),
                        );
                    }
                }
                Token::CurlyClose => {
                    break;
                }
                Token::Quotes | Token::ArrayClose => {}
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
                        value.insert(key.to_string(), Value::Array(Self::process_array(iterator)));
                        current_key = None;
                    }
                }
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

    /// Parse an array from token stream
    #[cfg_attr(feature = "profile", instrument)]
    fn process_array(iterator: &mut Peekable<Iter<Token>>) -> Vec<Value> {
        let mut internal_value = Vec::<Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => {
                    internal_value.push(Value::Object(Self::process_object(iterator)));
                }
                Token::String(string) => internal_value.push(Value::String(string.clone())),
                Token::Number(number) => internal_value.push(Value::Number(*number)),
                Token::ArrayOpen => {
                    internal_value.push(Value::Array(Self::process_array(iterator)));
                }
                Token::ArrayClose => {
                    break;
                }
                Token::Boolean(boolean) => internal_value.push(Value::Boolean(*boolean)),
                Token::Null => internal_value.push(Value::Null),
                Token::Comma | Token::CurlyClose | Token::Quotes | Token::Colon => {}
            }
        }

        internal_value
    }

    /// Create a new [`JsonParser`] that parses JSON from bytes.
    ///
    /// # Arguments
    ///
    /// * `input`: A slice of bytes from input data.
    ///
    /// returns: Result<Value, ()>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    #[cfg_attr(feature = "profile", instrument)]
    pub fn parse_from_bytes<'a>(input: &'a [u8]) -> Result<Value, ()> {
        let mut json_tokenizer = JsonTokenizer::<BufReader<Cursor<&[u8]>>>::from_bytes(input);
        let tokens = json_tokenizer.tokenize_json()?;

        Ok(Self::tokens_to_value(tokens))
    }

    #[cfg_attr(feature = "profile", instrument)]
    pub fn parse(reader: File) -> Result<Value, ()> {
        let mut json_tokenizer = JsonTokenizer::<BufReader<File>>::new(reader);
        let tokens = json_tokenizer.tokenize_json()?;

        Ok(Self::tokens_to_value(tokens))
    }

    #[cfg_attr(feature = "profile", instrument)]
    fn tokens_to_value(tokens: &[Token]) -> Value {
        let mut iterator = tokens.iter().peekable();

        let mut value = Value::Null;

        while let Some(token) = iterator.next() {
            match token {
                Token::CurlyOpen => {
                    value = Value::Object(Self::process_object(&mut iterator));
                }
                Token::String(string) => {
                    value = Value::String(string.clone());
                }
                Token::Number(number) => {
                    value = Value::Number(*number);
                }
                Token::ArrayOpen => {
                    value = Value::Array(Self::process_array(&mut iterator));
                }
                Token::Boolean(boolean) => value = Value::Boolean(*boolean),
                Token::Null => value = Value::Null,
                Token::Comma
                | Token::CurlyClose
                | Token::Quotes
                | Token::Colon
                | Token::ArrayClose => {}
            }
        }

        value
    }
}
