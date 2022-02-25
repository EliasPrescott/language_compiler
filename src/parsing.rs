pub struct ParseInput {
    pub position: usize,
    pub chars: Vec<ParsedChar>,
}

#[derive(Clone, Copy)]
pub struct ParseSavePoint(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct ParsedChar {
    pub char: char,
    pub line: u32,
    pub column: u32,
}

impl ToString for ParsedChar {
    // Implementing this allows parsing functions to quickly convert ParsedChar to char without worrying about the other fields.
    fn to_string(&self) -> String {
        self.char.to_string()
    }
}

impl ParsedChar {
    fn display_location(&self) -> String {
        format!("line: {}, column: {}", self.line, self.column)
    }
}

impl ParseInput {
    /// Gets an option of the next character without moving the cursor
    pub fn get_next_char(&self) -> Option<ParsedChar> {
        // Cloning the ParsedChar on every get isn't great, but it avoids lots of lifetime headaches.
        // It may be unavoidable, but there could be a way to work with refernces and juggle the lifetimes better.
        self.chars.get(self.position).map(|x| x.clone())
    }

    pub fn create_save_point(&self) -> ParseSavePoint {
        ParseSavePoint(self.position)
    }

    pub fn load_save_point(&mut self, ParseSavePoint(save_point): ParseSavePoint) {
        self.position = save_point;
    }

    pub fn get_next_char_result(&self) -> Result<ParsedChar, String> {
        match self.get_next_char() {
            Some(parsed_char) => Ok(parsed_char),
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn get_next_char_predicate(&self, predicate: &dyn Fn(ParsedChar) -> Result<ParsedChar, String>) -> Result<ParsedChar, String> {
        match self.get_next_char() {
            Some(parsed_char) => predicate(parsed_char),
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn get_next_char_alphabetical(&self) -> Result<ParsedChar, String> {
        self.get_next_char_predicate(&|parsed_char: ParsedChar| if parsed_char.char.is_alphabetic() { Ok(parsed_char) } else { Err(format!("Expected alphabetical character at {}, but found {}", parsed_char.display_location(), parsed_char.char)) })
    }

    pub fn get_next_char_numerical(&self) -> Result<ParsedChar, String> {
        self.get_next_char_predicate(&|parsed_char: ParsedChar| if parsed_char.char.is_numeric() { Ok(parsed_char) } else { Err(format!("Expected numerical character at {}, but found {}", parsed_char.display_location(), parsed_char.char)) })
    }

    pub fn get_next_char_alphabetical_or_in_group(&self, accepted_chars: &Vec<char>) -> Result<ParsedChar, String> {
        self.get_next_char_predicate(
            &|parsed_char: ParsedChar| {
                if parsed_char.char.is_alphabetic() || accepted_chars.contains(&parsed_char.char) {
                    Ok(parsed_char)
                } else { 
                    Err(format!("Expected alphabetical character or one of {:?} at {}, but found {}", accepted_chars, parsed_char.display_location(), parsed_char.char)) 
                }
            }
        )
    }

    /// Uses char indices now
    /// Gets an option of the next character and advances the cursor position by one
    pub fn pop_next_char(&mut self) -> Option<ParsedChar> {
        if let Some(parsed_char) = self.chars.get(self.position) {
            // Can't borrow self as mutable after immutable borrow by using self.skip_next_char(), but can modify directly?
            self.position += 1;
            Some(parsed_char.clone())
        } else {
            None
        }
    }

    pub fn pop_next_char_result(&mut self) -> Result<ParsedChar, String> {
        match self.get_next_char() {
            Some(parsed_char) => {
                self.skip_next_char();
                Ok(parsed_char.clone())
            },
            None => Err(String::from("Expected character, but found end of parser input"))
        }  
    }

    pub fn pop_next_char_predicate(&mut self, predicate: &dyn Fn(ParsedChar) -> Result<ParsedChar, String>) -> Result<ParsedChar, String> {
        match self.get_next_char() {
            Some(parsed_char) => {
                let x = predicate(parsed_char);
                if x.is_ok() {
                    self.skip_next_char();
                }
                x
            },
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn pop_next_char_alphabetical(&mut self) -> Result<ParsedChar, String> {
        self.pop_next_char_predicate(&|parsed_char: ParsedChar| if parsed_char.char.is_alphabetic() { Ok(parsed_char) } else { Err(format!("Expected alphabetical character at {}, but found {}", parsed_char.display_location(), parsed_char.char)) })
    }

    pub fn pop_next_char_numerical(&mut self) -> Result<ParsedChar, String> {
        self.pop_next_char_predicate(&|parsed_char: ParsedChar| if parsed_char.char.is_numeric() { Ok(parsed_char) } else { Err(format!("Expected numerical character at {}, but found {}", parsed_char.display_location(), parsed_char.char)) })
    }

    pub fn pop_next_char_alphabetical_or_in_group(&mut self, accepted_chars: &Vec<char>) -> Result<ParsedChar, String> {
        self.pop_next_char_predicate(
            &|parsed_char: ParsedChar| {
                if parsed_char.char.is_alphabetic() || accepted_chars.contains(&parsed_char.char) {
                    Ok(parsed_char)
                } else { 
                    Err(format!("Expected alphabetical character or one of {:?} at {}, but found {}", accepted_chars, parsed_char.display_location(), parsed_char.char)) 
                }
            }
        )
    }

    pub fn skip_next_char(&mut self) {
        if self.position + 1 < self.chars.len() {
            self.position += 1;
        } else {
            self.position = self.chars.len();
        }
    }

    /// Skips the next x number of characters
    pub fn skip_x_chars(&mut self, x: usize) {
        for _ in 0..x {
            if let Some(_) = self.chars.get(self.position + 1) {
                self.skip_next_char();
            }
        }
    }

    /// Uses char indices properly.
    /// Gets the text from the cursor position onwards
    pub fn get_remaining_text(&self) -> Result<String, String> {
        match self.chars.get(self.position..) {
            Some(parsed_chars) => {
                let mut output = String::new();
                for c in parsed_chars {
                    output += &c.char.to_string();
                }
                Ok(output)
            },
            None => {
                Err("Invalid input access".to_string())
            }
        }
    }

    /// Gets the next x number of characters without moving the cursor
    pub fn get_next_x_chars(&self, x: usize) -> Option<Vec<ParsedChar>> {
        let mut output = Vec::new();
        let mut current_index = self.position;
        for _ in self.position..x+self.position {
            if let Some(parsed_char) = self.chars.get(current_index) {
                output.push(parsed_char.clone());
                current_index += 1;
            } else {
                return None;
            }
        }
        Some(output)
    }

    /// Determines if the next block of characters is equal to the predicate string
    pub fn match_word(&self, predicate: &str) -> bool {
        // This is way too complicated, but it should work
        let input_string = 
            self.get_next_x_chars(predicate.len())
                .map(|parsed_chars|
                    parsed_chars.into_iter().map(|c| c.to_string())
                        .reduce(|a, b| a + &b)
                )
                .flatten();
        input_string == Some(predicate.to_string())
    }

    pub fn finished(&self) -> bool {
        self.get_next_char().is_none()
    }

    pub fn pop_char(&mut self, predicate: char) -> Result<ParsedChar, String> {
        match self.get_next_char() {
            Some(parsed_char) => {
                if parsed_char.char == predicate {
                    self.skip_next_char();
                    Ok(parsed_char)
                } else {
                    Err(format!("Expected: '{predicate}' at {}, but found '{}'", parsed_char.display_location(), parsed_char.char))
                }
            },
            None => Err(format!("Expected: '{predicate}', but found end of parse text"))
        }
    }

    /// Skips the cursor past an expected character, and returns an error message if the expected character is not found.
    pub fn skip_char(&mut self, predicate: char) -> Result<(), String> {
        match self.get_next_char() {
            Some(parsed_char) => {
                if parsed_char.char == predicate {
                    self.skip_next_char();
                    Ok(())
                } else {
                    Err(format!("Expected: '{predicate}' at {}, but found '{}'", parsed_char.display_location(), parsed_char.char))
                }
            },
            None => Err(format!("Expected: '{predicate}', but found end of parse text"))
        }
    }

    /// Skips the cursor past an expected string, and returns an error message if the expected string is not found.
    pub fn skip_string(&mut self, predicate: &str) -> Result<(), String> {
        if self.match_word(predicate) {
            self.skip_x_chars(predicate.len());
            Ok(())
        } else {
            if let Some(next_char) = self.get_next_char() {
                Err(format!("Expected keyword '{}' at, {}", predicate, next_char.display_location()))
            } else {
                Err(format!("Expected keyword '{}'", predicate))
            }
        }
    }

    pub fn skip_any_of_char(&mut self, skip_char: char) {
        while let Some(parsed_char) = self.get_next_char()  {
            self.skip_next_char();
        }
    }

    pub fn skip_any_of_chars(&mut self, skip_chars: Vec<char>) {
        loop {
            match self.get_next_char() {
                Some(parsed_char) => {
                    if skip_chars.contains(&parsed_char.char) {
                        self.skip_next_char()
                    } else {
                        break;
                    }
                },
                None => break,
            }
        }
    }

    pub fn skip_spaces(&mut self) {
        self.skip_any_of_char(' ');
    }

    pub fn skip_spaces_and_newlines(&mut self) {
        self.skip_any_of_chars(vec!(' ', '\n', '\r'));
    }

    pub fn pop_until_char(&mut self, stop_char: char) -> String {
        let mut output = "".to_string();
        while let Some(parsed_char) = self.get_next_char() {
            if parsed_char.char == stop_char {
                return output;
            } else {
                self.skip_next_char();
                output += &parsed_char.char.to_string();
            }
        }
        output
    }

    pub fn pop_until_chars(&mut self, stop_chars: Vec<char>) -> String {
        let mut output = "".to_string();
        while let Some(parsed_char) = self.get_next_char() {
            if stop_chars.contains(&parsed_char.char) {
                return output;
            } else {
                self.skip_next_char();
                output += &parsed_char.char.to_string();
            }
        }
        output
    }

    pub fn new(text: String) -> Self {
        let mut chars: Vec<ParsedChar> = Vec::new();
        let mut line: u32 = 1;
        for line_text in text.split('\n') {
            let mut column: u32 = 1;
            for char in line_text.chars() {
                chars.push(
                    ParsedChar {
                        char,
                        column,
                        line,
                    }
                );
                column += 1;
            }
            line += 1;
        }
        ParseInput {
            position: 0,
            chars,
        }
    }
}