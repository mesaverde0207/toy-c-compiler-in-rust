use std::cell::RefCell;
use std::iter::Peekable;
use std::process;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
enum TokenKind {
    TkReserved,
    TkIdent,
    TkKeyword,
    TkNum,
    TkEOF,
}

type TokenLink = Option<Rc<RefCell<Token>>>;

type PeekableString<'a> = Peekable<std::str::Chars<'a>>;

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    next: TokenLink,
    val: Option<String>,
    string: String, // token string
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    current: TokenLink,
    chars: Peekable<std::str::Chars<'a>>,
    head: TokenLink,
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(string: &'a str) -> Self {
        let mut tokenizer = Tokenizer::new_empty(string.chars().peekable());
        let mut next_char: Option<&char>;
        loop {
            next_char = tokenizer.chars.peek();
            match next_char {
                Some(' ') => {
                    tokenizer.chars.next();
                }
                // is there a better notation?
                Some('+') | Some('-') | Some('*') | Some('/') | Some('(') | Some(')')
                | Some(';') | Some('{') | Some('}') | Some('&') => {
                    let string = tokenizer.chars.next().unwrap().to_string();
                    tokenizer.new_token(TokenKind::TkReserved, string);
                }
                Some('=') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    let next_char = tokenizer.chars.peek();
                    match next_char {
                        Some(c) if c == &'=' => string.push(tokenizer.chars.next().unwrap()),
                        _ => {}
                    }
                    tokenizer.new_token(TokenKind::TkReserved, string);
                }
                Some('>') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    string = Tokenizer::peek_and_append_char(&mut tokenizer, string, '=');
                    tokenizer.new_token(TokenKind::TkReserved, string);
                }
                Some('<') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    string = Tokenizer::peek_and_append_char(&mut tokenizer, string, '=');
                    tokenizer.new_token(TokenKind::TkReserved, string);
                }
                Some('!') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    let next_char = tokenizer.chars.next().unwrap_or('\0');
                    if next_char != '=' {
                        eprintln!("tokenizer: unexpected token '!'");
                        process::exit(1);
                    } else {
                        string.push(next_char);
                    }
                    tokenizer.new_token(TokenKind::TkReserved, string);
                }
                Some('a'..='z') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    string = Tokenizer::parse_ident(&mut tokenizer, string);
                    tokenizer.new_token(TokenKind::TkIdent, string);
                }
                Some('_') => {
                    let mut string = tokenizer.chars.next().unwrap().to_string();
                    string = Tokenizer::parse_ident(&mut tokenizer, string);
                    tokenizer.new_token(TokenKind::TkIdent, string);
                }
                Some('0'..='9') => {
                    tokenizer.new_token(TokenKind::TkNum, String::from(""));
                }
                Some(_) => {
                    eprintln!("{}", string);
                    eprintln!("tokenizer: Not implemented");
                    process::exit(1);
                }
                None => {
                    tokenizer.new_token(TokenKind::TkEOF, String::from(""));
                    break;
                }
            }
        }

        tokenizer
    }

    fn new_empty(chars: PeekableString<'a>) -> Tokenizer {
        Tokenizer {
            current: None,
            chars: chars,
            head: None,
        }
    }

    fn new_token(&mut self, mut kind: TokenKind, string: String) {
        let mut val: Option<String> = None;
        if kind == TokenKind::TkNum {
            val = self.parse_int();
        } else if kind == TokenKind::TkIdent {
            kind = self.convert_keyword(&string)
        }

        let token = Token {
            kind: kind,
            next: None,
            val: val,
            string: string,
        };
        let token_pointer = Rc::new(RefCell::new(token));

        match self.current.take() {
            Some(curr) => {
                curr.borrow_mut().next = Some(token_pointer.clone());
            }
            None => {
                self.head = Some(token_pointer.clone());
            }
        }

        self.current = Some(token_pointer);
    }

    pub fn expect(&mut self, op: &str) {
        if let Some(head) = self.head.as_ref() {
            let head_ref = head.borrow();
            if head_ref.kind != TokenKind::TkReserved || head_ref.string != op {
                eprintln!("the character is not {}, got={:?}", op, head_ref.string);
                process::exit(1);
            }
        } else {
            eprintln!("head is None");
            process::exit(1);
        }

        self.head.take().map(|head| {
            if let Some(next) = head.borrow().next.clone() {
                self.head = Some(next);
            }
        });
    }

    pub fn consume(&mut self, op: &str) -> bool {
        if let Some(head) = self.head.as_ref() {
            let head_ref = head.borrow();
            if head_ref.kind != TokenKind::TkReserved && head_ref.kind != TokenKind::TkKeyword {
                return false;
            }
            if head_ref.string != op.to_string() {
                return false;
            }
        } else {
            eprintln!("head is None");
            process::exit(1);
        }

        self.head.take().map(|head| {
            if let Some(next) = head.borrow().next.clone() {
                self.head = Some(next);
            }
        });
        return true;
    }

    pub fn expect_number(&mut self) -> Option<String> {
        let val = self.head.take().map(|head| {
            let mut head_ref = head.borrow_mut();
            if let Some(next) = head_ref.next.take() {
                if head_ref.kind != TokenKind::TkNum {
                    eprintln!("Not a number");
                    process::exit(1);
                }
                self.head = Some(next);
                head_ref.val.clone()
            } else {
                None
            }
        });
        return val.unwrap_or(Some(String::from("")));
    }

    pub fn is_ident_token(&mut self) -> Option<String> {
        let string: Option<String>;

        if let Some(head) = self.head.clone() {
            let head_ref = head.borrow();
            if head_ref.kind == TokenKind::TkIdent {
                string = Some(head_ref.string.clone());
                self.head.take().map(|head| {
                    if let Some(next) = head.borrow().next.clone() {
                        self.head = Some(next);
                    }
                });
            } else {
                string = None;
            }
        } else {
            eprintln!("lexer: tokenizer's head is None");
            process::exit(1);
        }

        string
    }

    pub fn at_eof(&mut self) -> bool {
        if let Some(ref head) = self.head {
            return head.borrow().kind == TokenKind::TkEOF;
        } else {
            eprintln!("tokenizer's head is None");
            process::exit(1);
        }
    }

    fn parse_int(&mut self) -> Option<String> {
        let mut integer = String::from("");

        while let Some(next_char) = self.chars.peek() {
            if next_char.is_numeric() {
                integer.push(self.chars.next().unwrap())
            } else if next_char == &' ' {
                self.chars.next();
                continue;
            } else {
                break;
            }
        }

        return Some(integer);
    }

    fn peek_and_append_char(
        tokenizer: &mut Tokenizer,
        mut string: String,
        expected: char,
    ) -> String {
        let next_char = tokenizer.chars.peek();
        match next_char {
            Some(c) if c == &expected => string.push(tokenizer.chars.next().unwrap()),
            None => {}
            _ => eprintln!("tokenizer: not implemented operator"),
        }
        return string;
    }

    fn parse_ident(tokenizer: &mut Tokenizer, mut string: String) -> String {
        loop {
            let next_char = tokenizer.chars.peek();
            match next_char {
                Some('a'..='z') => string.push(tokenizer.chars.next().unwrap()),
                Some('A'..='Z') => string.push(tokenizer.chars.next().unwrap()),
                Some('0'..='9') => string.push(tokenizer.chars.next().unwrap()),
                Some('_') => string.push(tokenizer.chars.next().unwrap()),
                None => break,
                _ => break,
            }
        }
        return string;
    }

    fn convert_keyword(&self, string: &str) -> TokenKind {
        let keywords = vec!["return", "if", "else", "for", "while"];

        for kw in keywords {
            if string == kw {
                return TokenKind::TkKeyword;
            }
        }
        return TokenKind::TkIdent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_int() {
        let mut tokenizer = Tokenizer::new_empty("42".chars().peekable());
        assert_eq!(tokenizer.parse_int().unwrap(), String::from("42"));
    }
}
