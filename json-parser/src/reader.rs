use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str::from_utf8;

/// A struct that handles reading input data to be parsed and provides an iterator over said data
/// character-by-character.
pub struct JsonReader {
    /// A reference to the input data, which can be anything that implements [`Read`]
    reader: Box<BufReader<dyn Read>>,
    /// A character buffer that holds queue of characters to be used by the iterator.
    ///
    /// This is necessary because UTF-8 can be 1-4 bytes long. Because of this, the reader
    /// always reads 4 bytes at a time. We then iterate over "characters", irrespective of whether
    /// they are 1 byte long, or 4.
    ///
    /// A [`VecDeque`] is used instead of a normal vector because characters need to be read out
    /// from the start of the buffer.
    character_buffer: VecDeque<char>,
}

impl JsonReader {
    /// Create a new [`JsonReader`] that reads from a file
    ///
    /// # Arguments
    ///
    /// * `reader`: An instance of buffered reader over file to be parsed.
    ///
    /// returns: JsonReader
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use json_parser::reader::JsonReader;
    ///
    /// let file = File::create("dummy.json")?;
    /// let reader = BufReader::new(file);
    ///
    /// let json_reader = JsonReader::new(reader);
    /// ```
    pub fn new(reader: BufReader<File>) -> Self {
        JsonReader {
            reader: Box::new(reader),
            character_buffer: VecDeque::with_capacity(4),
        }
    }

    /// Create a new [`JsonReader`] that reads from a given byte stream
    ///
    /// # Arguments
    ///
    /// * `reader`: An instance of buffered reader over input string to be parsed.
    ///
    /// returns: JsonReader
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::BufReader;
    /// use json_parser::reader::JsonReader;
    ///
    /// let input_json_string = r#"{"key1":"value1","key2":"value2"}"#;
    /// let reader = BufReader::new(input_json_string.as_bytes());
    ///
    /// let json_reader = JsonReader::from_bytes(reader);
    /// ```
    pub fn from_bytes(reader: BufReader<&'static [u8]>) -> Self {
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
            // TODO: Read "valid_upto" from error and rewind read buffer by errored bytes count
            None
        }
    }
}
