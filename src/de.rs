use std::collections::HashMap;

use crate::token;

#[derive(Eq, PartialEq, Debug)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

pub struct Deserializer<'a> {
    tokenizer: token::Tokenizer<'a>,
}

impl Deserializer<'_> {
    pub fn new(input: &str) -> Deserializer {
        Deserializer {
            tokenizer: token::Tokenizer::new(input),
        }
    }

    pub fn parse(&mut self) -> Result<Option<Value>, token::Error> {
        self.value()
    }

    fn value(&mut self) -> Result<Option<Value>, token::Error> {
        self.tokenizer.eat_whitespaces();

        match self.tokenizer.next()? {
            Some(token::Token::LeftBrace) => self.object(),
            Some(token::Token::LeftBracket) => self.array(),
            Some(token::Token::String(string)) => Ok(Some(Value::String(string))),
            Some(token::Token::Number(string)) => Ok(Some(Value::Number(string))),
            Some(token::Token::Bool(boolean)) => Ok(Some(Value::Bool(boolean))),
            Some(token::Token::Null) => Ok(Some(Value::Null)),
            _ => Err(token::Error::InvalidToken),
        }
    }

    fn object(&mut self) -> Result<Option<Value>, token::Error> {
        let mut object = HashMap::new();

        // empty object
        if self.tokenizer.eat_token(token::Token::RightBrace) {
            return Ok(Some(Value::Object(object)));
        }

        loop {
            self.tokenizer.eat_whitespaces();

            match self.tokenizer.next()? {
                Some(token::Token::String(key)) => {
                    self.tokenizer.eat_whitespaces();

                    // :
                    if !self.tokenizer.eat_token(token::Token::Colon) {
                        break;
                    }

                    self.tokenizer.eat_whitespaces();

                    // value
                    match self.value()? {
                        Some(value) => {
                            object.insert(key, value);
                        }
                        _ => break,
                    }

                    self.tokenizer.eat_whitespaces();

                    // }
                    if self.tokenizer.eat_token(token::Token::RightBrace) {
                        return Ok(Some(Value::Object(object)));
                    }

                    // ,
                    if !self.tokenizer.eat_token(token::Token::Comma) {
                        break;
                    }
                }
                _ => break,
            }
        }

        Err(token::Error::InvalidToken)
    }

    fn array(&mut self) -> Result<Option<Value>, token::Error> {
        let mut array = vec![];

        self.tokenizer.eat_whitespaces();

        // empty array
        if self.tokenizer.eat_token(token::Token::RightBracket) {
            return Ok(Some(Value::Array(array)));
        }

        loop {
            self.tokenizer.eat_whitespaces();

            match self.value()? {
                Some(value) => array.push(value),
                _ => return Err(token::Error::InvalidToken),
            }

            self.tokenizer.eat_whitespaces();

            // ]
            if self.tokenizer.eat_token(token::Token::RightBracket) {
                return Ok(Some(Value::Array(array)));
            }

            // ,
            if !self.tokenizer.eat_token(token::Token::Comma) {
                break;
            }
        }

        Err(token::Error::InvalidToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserializer_parse() {
        // simple object
        let input = r#"
{
  "name": "jjsonsonpapaparser",
  "desc": "toy json parser",
  "true_key": true,
  "false_key": false,
  "null_key": null
}
"#;
        let mut deserializer = Deserializer::new(input);
        match deserializer.parse().unwrap() {
            Some(Value::Object(object)) => {
                assert_eq!(
                    object["name"],
                    Value::String("jjsonsonpapaparser".to_string())
                );
                assert_eq!(object["desc"], Value::String("toy json parser".to_string()));
                assert_eq!(object.get("missing_key"), None);
            }
            _ => panic!("Should be Object"),
        }

        // nested object
        let input = r#"
{
  "name": {
    "nested_name": "nested_name_value",
    "nested_desc": "nested_desc_value"
  },
  "desc": "toy json parser"
}
"#;
        let mut deserializer = Deserializer::new(input);
        match deserializer.parse().unwrap() {
            Some(Value::Object(object)) => {
                match object.get("name").unwrap() {
                    Value::Object(nested_object) => {
                        assert_eq!(
                            nested_object["nested_name"],
                            Value::String("nested_name_value".to_string())
                        );
                        assert_eq!(
                            nested_object["nested_desc"],
                            Value::String("nested_desc_value".to_string())
                        );
                    }
                    _ => panic!("Should be nested Object"),
                }
                assert_eq!(object["desc"], Value::String("toy json parser".to_string()));
                assert_eq!(object.get("missing_key"), None);
            }
            _ => panic!("Should be Object"),
        }

        // array
        let input = r#"
[
  1,
  2,
  true,
  "abcd"
]
"#;
        let mut deserializer = Deserializer::new(input);
        match deserializer.parse().unwrap() {
            Some(Value::Array(array)) => {
                assert_eq!(array[0], Value::Number("1".to_string()));
                assert_eq!(array[1], Value::Number("2".to_string()));
                assert_eq!(array[2], Value::Bool(true));
                assert_eq!(array[3], Value::String("abcd".to_string()));
            }
            _ => panic!("Should be Array"),
        }
    }
}
