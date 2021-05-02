/// Sample Json
/// {
///   "name": "jjsonsonpapaparser",
///   "desc": "toy json parser",
/// }
use std::str;
use std::string;

#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Number(string::String),
    Integer(string::String),
    Fraction(string::String),
    Exponent(string::String),
    String(string::String),
    Bool(bool),
    Null,
}

impl Token {
    fn to_char(self) -> Option<char> {
        match self {
            Token::LeftBracket => Some('['),
            Token::RightBracket => Some(']'),
            Token::LeftBrace => Some('{'),
            Token::RightBrace => Some('}'),
            Token::Comma => Some(','),
            Token::Colon => Some(':'),
            _ => None,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
    SomethingError,
    EOF,
    InvalidEscapeChar,
    InvalidString,
    InvalidNumber,
    InvalidToken,
}

/// Token parser
pub struct Tokenizer<'a> {
    chars: str::CharIndices<'a>,
}

impl Tokenizer<'_> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer {
            chars: input.char_indices(),
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>, Error> {
        match self.peek() {
            Some((_, '{')) => {
                self.eat_one();
                Ok(Some(Token::LeftBrace))
            }
            Some((_, '}')) => {
                self.eat_one();
                Ok(Some(Token::RightBrace))
            }
            Some((_, '[')) => {
                self.eat_one();
                Ok(Some(Token::LeftBracket))
            }
            Some((_, ']')) => {
                self.eat_one();
                Ok(Some(Token::RightBracket))
            }
            Some((_, ',')) => {
                self.eat_one();
                Ok(Some(Token::Comma))
            }
            Some((_, ':')) => {
                self.eat_one();
                Ok(Some(Token::Colon))
            }
            Some((_, '"')) => self.string(),
            Some((_, c)) if c.is_digit(10) || c == '-' => self.number(),
            Some((_, c)) if c == 't' || c == 'f' => self.boolean(),
            Some((_, 'n')) => self.null(),
            Some(_) => Err(Error::InvalidToken),
            None => Err(Error::EOF),
        }
    }

    pub fn eat_token(&mut self, token: Token) -> bool {
        match token.to_char() {
            Some(c) => self.eatc(c),
            _ => false,
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        self.chars.clone().next()
    }

    fn one(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn eat_one(&mut self) -> bool {
        match self.one() {
            Some(_) => true,
            _ => false,
        }
    }

    fn eatc(&mut self, c: char) -> bool {
        match self.peek() {
            Some((_, val)) if c == val => {
                self.eat_one();
                true
            }
            _ => false,
        }
    }

    fn eats(&mut self, s: &str) -> bool {
        let mut chars = self.chars.clone();
        let mut i = 0;

        for c in s.chars() {
            i += 1;

            if let Some((_, ch)) = chars.next() {
                if c == ch {
                    continue;
                }
            }

            return false;
        }

        for _ in 0..i {
            self.one();
        }
        true
    }

    /// "whitespace" are space( ), linefeed(\n), carriage return(\r) and horizontal tab(\t)
    fn eat_whitespace(&mut self) -> bool {
        self.eatc(' ') || self.eatc('\n') || self.eatc('\r') || self.eatc('\t')
    }

    pub fn eat_whitespaces(&mut self) -> bool {
        if !self.eat_whitespace() {
            return false;
        }

        while self.eat_whitespace() {}
        true
    }

    /// Assumes that head quotation mark(") have been already eaten.
    fn string(&mut self) -> Result<Option<Token>, Error> {
        let mut val = "".to_string();

        match self.peek() {
            Some((_, '"')) => self.one(),
            _ => return Err(Error::InvalidString),
        };

        loop {
            match self.peek() {
                Some((_, '\\')) => {
                    self.one();
                    match self.peek() {
                        Some((_, '"')) => {
                            self.one();
                            val.push('"');
                        }
                        Some((_, '\\')) => {
                            self.one();
                            val.push('\\');
                        }
                        Some((_, '/')) => {
                            self.one();
                            val.push('/');
                        }
                        Some((_, 'b')) => {
                            self.one();
                            val.push('\x08')
                        }
                        Some((_, 'f')) => {
                            self.one();
                            val.push('\x0C');
                        }
                        Some((_, 'n')) => {
                            self.one();
                            val.push('\n');
                        }
                        Some((_, 'r')) => {
                            self.one();
                            val.push('\x0D');
                        }
                        Some((_, 't')) => {
                            self.one();
                            val.push('\t');
                        }
                        // TODO: implement `u`
                        _ => return Err(Error::InvalidEscapeChar),
                    }
                }
                Some((_, '"')) => {
                    self.one();
                    return Ok(Some(Token::String(val)));
                }
                Some((_, c)) => {
                    self.one();
                    val.push(c);
                }
                None => return Err(Error::EOF),
            }
        }
    }

    /// - 100
    /// - 0
    /// - 0.001e-10
    fn number(&mut self) -> Result<Option<Token>, Error> {
        let mut val = "".to_string();

        // Integer
        match self.integer() {
            Ok(Some(Token::Integer(n))) => val.push_str(n.as_str()),
            _ => return Err(Error::InvalidNumber),
        }

        // Fraction
        match self.peek() {
            Some((_, '.')) => {
                self.one();
                val.push('.');
            }
            _ => return Ok(Some(Token::Number(val))),
        }
        match self.fraction() {
            Ok(Some(Token::Fraction(n))) => val.push_str(n.as_str()),
            _ => return Err(Error::InvalidNumber),
        }

        // Exponent
        match self.peek() {
            Some((_, 'e')) => {
                self.one();
                val.push('e');
            }
            Some((_, 'E')) => {
                self.one();
                val.push('E');
            }
            _ => return Ok(Some(Token::Number(val))),
        }
        match self.exponent() {
            Ok(Some(Token::Exponent(n))) => val.push_str(n.as_str()),
            _ => return Err(Error::InvalidNumber),
        }

        return Ok(Some(Token::Number(val)));
    }

    fn integer(&mut self) -> Result<Option<Token>, Error> {
        let mut val = "".to_string();

        // -
        if let Some((_, '-')) = self.peek() {
            self.eat_one();
            val.push('-');
        }

        // 0
        if let Some((_, '0')) = self.peek() {
            self.eat_one();
            val.push('0');

            return Ok(Some(Token::Integer(val)));
        }

        loop {
            match self.peek() {
                Some((_, c)) if c.is_digit(10) => {
                    self.eat_one();
                    val.push(c);
                }
                _ => return Ok(Some(Token::Integer(val))),
            };
        }
    }

    fn fraction(&mut self) -> Result<Option<Token>, Error> {
        let mut val = "".to_string();

        if let Some((_, c)) = self.peek() {
            if c.is_digit(10) {
                self.one();
                val.push(c);
            } else {
                return Err(Error::InvalidNumber);
            }
        }

        loop {
            if let Some((_, c)) = self.peek() {
                if c.is_digit(10) {
                    self.eat_one();
                    val.push(c);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        return Ok(Some(Token::Fraction(val)));
    }

    fn exponent(&mut self) -> Result<Option<Token>, Error> {
        let mut val = "".to_string();

        if let Some((_, c)) = self.peek() {
            if c == '-' || c == '+' {
                self.one();
                val.push(c);
            }
        }

        match self.fraction() {
            Ok(Some(Token::Fraction(f))) => {
                val.push_str(f.as_str());
                return Ok(Some(Token::Exponent(val)));
            }
            _ => return Err(Error::InvalidNumber),
        }
    }

    fn boolean(&mut self) -> Result<Option<Token>, Error> {
        if self.eats("true") {
            Ok(Some(Token::Bool(true)))
        } else if self.eats("false") {
            Ok(Some(Token::Bool(false)))
        } else {
            Err(Error::InvalidToken)
        }
    }

    fn null(&mut self) -> Result<Option<Token>, Error> {
        if self.eats("null") {
            Ok(Some(Token::Null))
        } else {
            Err(Error::InvalidToken)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_next() {
        let input = r#"
{
  "name": "jjsonsonpapaparser",
  "desc": "toy json parser",
}
"#;
        let mut tokenizer = Tokenizer::new(input);

        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::LeftBrace)));
        tokenizer.eat_whitespaces();
        assert_eq!(
            tokenizer.next(),
            Ok(Some(Token::String("name".to_string())))
        );
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::Colon)));
        tokenizer.eat_whitespaces();
        assert_eq!(
            tokenizer.next(),
            Ok(Some(Token::String("jjsonsonpapaparser".to_string())))
        );
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::Comma)));
        tokenizer.eat_whitespaces();
        assert_eq!(
            tokenizer.next(),
            Ok(Some(Token::String("desc".to_string())))
        );
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::Colon)));
        tokenizer.eat_whitespaces();
        assert_eq!(
            tokenizer.next(),
            Ok(Some(Token::String("toy json parser".to_string())))
        );
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::Comma)));
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Ok(Some(Token::RightBrace)));
        tokenizer.eat_whitespaces();
        assert_eq!(tokenizer.next(), Err(Error::EOF));
    }

    #[test]
    fn test_tokenizer_peek() {
        let input = "abcd";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(tokenizer.peek(), Some((0, 'a')));
        assert_eq!(tokenizer.peek(), Some((0, 'a')));
        assert_eq!(tokenizer.peek(), Some((0, 'a')));
    }

    #[test]
    fn test_tokenizer_one() {
        let input = "abc";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(tokenizer.one(), Some((0, 'a')));
        assert_eq!(tokenizer.one(), Some((1, 'b')));
        assert_eq!(tokenizer.one(), Some((2, 'c')));
        assert_eq!(tokenizer.one(), None);
    }

    #[test]
    fn test_tokenizer_eat_one() {
        let input = "abc";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert!(tokenizer.eat_one());
        assert!(tokenizer.eat_one());
        assert!(tokenizer.eat_one());
        assert!(!tokenizer.eat_one());
    }

    #[test]
    fn test_tokenizer_eatc() {
        let input = "abc";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert!(tokenizer.eatc('a'));
        assert!(tokenizer.eatc('b'));
        assert!(!tokenizer.eatc('d'));
        assert!(!tokenizer.eatc('e'));
        assert!(tokenizer.eatc('c'));
        assert!(!tokenizer.eatc('c'));
    }

    #[test]
    fn test_tokenizer_eats() {
        let input = "true";
        let mut tokenizer = Tokenizer::new(input);
        assert!(tokenizer.eats("true"));
        assert_eq!(tokenizer.next(), Err(Error::EOF));

        let input = "truehoge";
        let mut tokenizer = Tokenizer::new(input);
        assert!(tokenizer.eats("true"));
        assert_eq!(tokenizer.next(), Err(Error::InvalidToken));
        assert_eq!(tokenizer.peek(), Some((4, 'h')));

        let input = "asdftruehoge";
        let mut tokenizer = Tokenizer::new(input);
        assert!(!tokenizer.eats("true"));
        assert_eq!(tokenizer.peek(), Some((0, 'a')));
    }

    #[test]
    fn test_tokenizer_string() {
        let input = "\"abcde  f \"";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.string(),
            Ok(Some(Token::String("abcde  f ".to_string())))
        );
    }

    #[test]
    fn test_tokenizer_number() {
        let input = "100";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.number(),
            Ok(Some(Token::Number(input.to_string())))
        );

        let input = "-100";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.number(),
            Ok(Some(Token::Number(input.to_string())))
        );

        let input = "-100.000";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.number(),
            Ok(Some(Token::Number(input.to_string())))
        );

        let input = "-100.001e10";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.number(),
            Ok(Some(Token::Number(input.to_string())))
        );
    }

    #[test]
    fn test_tokenizer_integer() {
        let input = "100";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.integer(),
            Ok(Some(Token::Integer(input.to_string())))
        );

        let input = "001";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.integer(),
            Ok(Some(Token::Integer('0'.to_string())))
        );
    }

    #[test]
    fn test_tokenizer_fraction() {
        let input = "100";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.fraction(),
            Ok(Some(Token::Fraction(input.to_string())))
        );

        let input = "010";
        let mut tokenizer = Tokenizer {
            chars: input.char_indices(),
        };

        assert_eq!(
            tokenizer.fraction(),
            Ok(Some(Token::Fraction(input.to_string())))
        );
    }
}
