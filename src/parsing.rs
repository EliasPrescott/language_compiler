pub struct ParseInput {
    pub position: usize,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct ParseSavePoint(usize);

pub type CharIndice = (usize, char);

impl ParseInput {
    /// Gets an option of the next character without moving the cursor
    pub fn get_next_char(&self) -> Option<CharIndice> {
        self.text.char_indices().into_iter().nth(self.position)
    }

    pub fn create_save_point(&self) -> ParseSavePoint {
        ParseSavePoint(self.position)
    }

    pub fn load_save_point(&mut self, ParseSavePoint(save_point): ParseSavePoint) {
        self.position = save_point;
    }

    pub fn get_next_char_result(&self) -> Result<char, String> {
        match self.get_next_char() {
            Some((_, char)) => Ok(char),
            None => Err(String::from("Expected character, but found end of parser input"))
        }
    }

    pub fn get_next_char_predicate(&self, predicate: &dyn Fn(char) -> Result<char, String>) -> Result<char, String> {
        match self.get_next_char() {
            Some((_, char)) => predicate(char),
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

    /// Uses char indices now
    /// Gets an option of the next character and advances the cursor position by one
    pub fn pop_next_char(&mut self) -> Option<char> {
        if let Some((_, char)) = self.text.char_indices().into_iter().nth(self.position) {
            if let Some((index, _)) = self.text.char_indices().into_iter().nth(self.position + 1) {
                self.position = index;
            }
            Some(char)
        } else {
            None
        }
    }

    pub fn pop_next_char_result(&mut self) -> Result<char, String> {
        match self.get_next_char() {
            Some((index, char)) => {
                self.position += 1;
                Ok(char)
            },
            None => Err(String::from("Expected character, but found end of parser input"))
        }  
    }

    pub fn pop_next_char_predicate(&mut self, predicate: &dyn Fn(char) -> Result<char, String>) -> Result<char, String> {
        match self.get_next_char() {
            Some((index, char)) => {
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

    /// Uses char indices properly
    pub fn skip_next_char(&mut self) {
        if let Some((index, _)) = self.text.char_indices().nth(self.position + 1) {
            self.position = index;
        } else {
            self.position = self.text.char_indices().collect::<Vec<(usize, char)>>().len();
        }
    }

    /// Skips the next x number of characters
    pub fn skip_x_chars(&mut self, x: usize) {
        for _ in 0..x {
            if let Some((index, _)) = self.text.char_indices().nth(self.position + 1) {
                self.position = index;
            }
        }
    }

    /// Uses char indices properly.
    /// Gets the text from the cursor position onwards
    pub fn get_remaining_text(&self) -> Result<String, String> {
        match self.text.chars().collect::<Vec<char>>().get(self.position..) {
            Some(chars) => {
                let mut output = String::new();
                for c in chars {
                    output += &c.to_string();
                }
                Ok(output)
            },
            None => {
                Err("Invalid input access".to_string())
            }
        }
    }

    /// Gets the next x number of characters without moving the cursor
    pub fn get_next_x_chars(&self, x: usize) -> Option<String> {
        let mut output = String::new();
        let mut current_index = self.position;
        for _ in self.position..x+self.position {
            if let Some((_, c)) = self.text.char_indices().nth(current_index) {
                output += &c.to_string();
                current_index += 1;
            } else {
                return None;
            }
        }
        Some(output)
    }

    /// Determines if the next block of characters is equal to the predicate string
    pub fn match_word(&self, predicate: &str) -> bool {
        self.get_next_x_chars(predicate.len()) == Some(predicate.to_string())
    }

    pub fn finished(&self) -> bool {
        self.get_next_char().is_none()
    }

    /// Skips the cursor past an expected character, and returns an error message if the expected character is not found.
    pub fn skip_char(&mut self, predicate: char) -> Result<(), String> {
        match self.get_next_char() {
            Some((_, next_char)) => {
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
            Err(format!("Expected keyword '{}'", predicate))
        }
    }

    pub fn skip_any_of_char(&mut self, skip_char: char) {
        while let Some((_, skip_char)) = self.get_next_char()  {
            self.skip_next_char();
        }
    }

    pub fn skip_any_of_chars(&mut self, skip_chars: Vec<char>) {
        loop {
            match self.get_next_char() {
                Some((index, next_char)) => {
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
        while let Some((index, next_char)) = self.get_next_char() {
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
        while let Some((index, next_char)) = self.get_next_char() {
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