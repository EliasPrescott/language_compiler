pub struct ParseInput {
    pub position: usize,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct ParseSavePoint(usize);

impl ParseInput {
    /// Gets an option of the next character without moving the cursor
    pub fn get_next_char(&self) -> Option<char> {
        self.text.chars().into_iter().nth(self.position)
    }

    pub fn create_save_point(&self) -> ParseSavePoint {
        ParseSavePoint(self.position)
    }

    pub fn load_save_point(&mut self, ParseSavePoint(save_point): ParseSavePoint) {
        self.position = save_point;
    }

    pub fn get_next_char_result(&self) -> Result<char, String> {
        match self.text.chars().into_iter().nth(self.position) {
            Some(char) => Ok(char),
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn get_next_char_predicate(&self, predicate: &dyn Fn(char) -> Result<char, String>) -> Result<char, String> {
        match self.text.chars().into_iter().nth(self.position) {
            Some(char) => predicate(char),
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn get_next_char_alphabetical(&self) -> Result<char, String> {
        self.get_next_char_predicate(&|char: char| if char.is_alphabetic() { Ok(char) } else { Err(format!("Expected alphabetical character, but found {}", char)) })
    }

    pub fn get_next_char_numerical(&self) -> Result<char, String> {
        self.get_next_char_predicate(&|char: char| if char.is_numeric() { Ok(char) } else { Err(format!("Expected numerical character, but found {}", char)) })
    }

    pub fn get_next_char_alphabetical_or_in_group(&self, accepted_chars: &Vec<char>) -> Result<char, String> {
        self.get_next_char_predicate(
            &|char: char| {
                if char.is_alphabetic() || accepted_chars.contains(&char) {
                    Ok(char)
                } else { 
                    Err(format!("Expected alphabetical character or one of {:?}, but found {}", accepted_chars, char)) 
                }
            }
        )
    }

    /// Gets an option of the next character and advances the cursor position by one
    pub fn pop_next_char(&mut self) -> Option<char> {
        match self.get_next_char() {
            Some(char) => {
                self.position += 1;
                Some(char)
            },
            None => None
        }        
    }

    pub fn pop_next_char_result(&mut self) -> Result<char, String> {
        match self.get_next_char() {
            Some(char) => {
                self.position += 1;
                Ok(char)
            },
            None => Err(String::from("Expected character, but found end of parser input"))
        }  
    }

    pub fn pop_next_char_predicate(&mut self, predicate: &dyn Fn(char) -> Result<char, String>) -> Result<char, String> {
        match self.get_next_char() {
            Some(char) => {
                let x = predicate(char);
                if x.is_ok() {
                    self.position += 1;
                }
                x
            },
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn pop_next_char_alphabetical(&mut self) -> Result<char, String> {
        self.pop_next_char_predicate(&|char: char| if char.is_alphabetic() { Ok(char) } else { Err(format!("Expected alphabetical character, but found {}", char)) })
    }

    pub fn pop_next_char_numerical(&mut self) -> Result<char, String> {
        self.pop_next_char_predicate(&|char: char| if char.is_numeric() { Ok(char) } else { Err(format!("Expected numerical character, but found {}", char)) })
    }

    pub fn pop_next_char_alphabetical_or_in_group(&mut self, accepted_chars: &Vec<char>) -> Result<char, String> {
        self.pop_next_char_predicate(
            &|char: char| {
                if char.is_alphabetic() || accepted_chars.contains(&char) {
                    Ok(char)
                } else { 
                    Err(format!("Expected alphabetical character or one of {:?}, but found {}", accepted_chars, char)) 
                }
            }
        )
    }

    /// Advances the cursor position by one. Should only be used when the next character is known.
    pub fn skip_next_char(&mut self) {
        self.position += 1;
    }

    /// Skips the next x number of characters
    pub fn skip_x_chars(&mut self, x: usize) {
        self.position += x;       
    }

    /// Gets the text from the cursor position onwards
    pub fn get_remaining_text(&self) -> &str {
        &self.text[self.position..]
    }

    /// Gets the next x number of characters without moving the cursor
    pub fn get_next_x_chars(&self, x: usize) -> Option<&str> {
        self.text.get(self.position..x)
    }

    /// Determines if the next block of characters is equal to the predicate string
    pub fn match_word(&self, predicate: &str) -> bool {
        self.get_next_x_chars(predicate.len()) == Some(predicate)
    }

    pub fn finished(&self) -> bool {
        self.get_next_char().is_none()
    }

    /// Skips the cursor past an expected character, and returns an error message if the expected character is not found.
    pub fn skip_char(&mut self, predicate: char) -> Result<(), String> {
        match self.get_next_char() {
            Some(next_char) => {
                if next_char == predicate {
                    self.skip_next_char();
                    Ok(())
                } else {
                    Err(format!("Expected: '{predicate}', but found '{next_char}'"))
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

            Err(format!("Expected: keyword '{}'", predicate))
        }
    }

    pub fn skip_any_of_char(&mut self, skip_char: char) {
        while self.get_next_char() == Some(skip_char) {
            self.skip_next_char();
        }
    }

    pub fn skip_any_of_chars(&mut self, skip_chars: Vec<char>) {
        loop {
            match self.get_next_char() {
                Some(next_char) => {
                    if skip_chars.contains(&next_char) {
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
        while let Some(next_char) = self.get_next_char() {
            if next_char == stop_char {
                return output;
            } else {
                self.skip_next_char();
                output += &next_char.to_string();
            }
        }
        output
    }

    pub fn pop_until_chars(&mut self, stop_chars: Vec<char>) -> String {
        let mut output = "".to_string();
        while let Some(next_char) = self.get_next_char() {
            if stop_chars.contains(&next_char) {
                return output;
            } else {
                self.skip_next_char();
                output += &next_char.to_string();
            }
        }
        output
    }

    pub fn new(text: String) -> Self {
        ParseInput {
            position: 0,
            text,
        }
    }
}