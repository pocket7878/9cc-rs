mod token;
mod token_list;
use std::process::exit;

use crate::error_report::error_at;

pub use self::{
    token::{Token, TokenKind},
    token_list::TokenList,
};

pub struct Tokenizer<'a> {
    original_input: &'a str,
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Self {
            original_input: input,
            input,
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> TokenList<'a> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();

            if self.input.is_empty() {
                break;
            }

            let current_position = self.pos;

            // Try tokenize syntax items
            let reserved_tokens = vec![
                ("<=", TokenKind::LessThanOrEqual),
                (">=", TokenKind::GreaterThanOrEqual),
                ("==", TokenKind::Equal),
                ("!=", TokenKind::NotEqual),
                (">", TokenKind::GreaterThan),
                ("<", TokenKind::LessThan),
                ("+", TokenKind::Plus),
                ("-", TokenKind::Minus),
                ("*", TokenKind::Mul),
                ("/", TokenKind::Div),
                ("(", TokenKind::LParen),
                (")", TokenKind::RParen),
                (";", TokenKind::Semicolon),
                ("=", TokenKind::Assign),
            ];
            let consumed_syntax_item = reserved_tokens
                .into_iter()
                .find(|(op, _)| self.try_consume(op));
            if let Some((_, kind)) = consumed_syntax_item {
                tokens.push(Token::new_syntax_item(current_position, kind));
                continue;
            }

            if let Some(num) = self.try_consume_digits() {
                tokens.push(Token::new_num(current_position, num));
                continue;
            }

            if let Some(c) = self.try_consume_alphabetic_char() {
                tokens.push(Token::new_ident(current_position, &c.to_string()));
                continue;
            }

            // 単純化のため、トークン化できなかったら即終了させる
            error_at(self.original_input, current_position, "Unrecognized token");
            exit(1)
        }

        TokenList::new(self.original_input, tokens)
    }

    fn skip_whitespace(&mut self) {
        use itertools::Itertools;

        let mut chars = self.input.chars();
        let spaces = chars
            .take_while_ref(|c| c.is_whitespace())
            .collect::<String>();
        self.pos += spaces.chars().count();
        self.input = chars.as_str();
    }

    fn try_consume(&mut self, str: &str) -> bool {
        if self.input.starts_with(&str) {
            self.pos += str.chars().count();
            self.input = &self.input[str.chars().count()..];
            true
        } else {
            false
        }
    }

    fn try_consume_digits(&mut self) -> Option<i32> {
        let first_non_num = self
            .input
            .find(|c| !char::is_numeric(c))
            .unwrap_or(self.input.len());
        let (digit_str, rest_input) = self.input.split_at(first_non_num);

        match digit_str.parse::<i32>() {
            Ok(num) => {
                self.input = rest_input;
                self.pos += digit_str.chars().count();
                Some(num)
            }
            Err(_) => None,
        }
    }

    fn try_consume_alphabetic_char(&mut self) -> Option<char> {
        let first_char = self.input.chars().next();
        if first_char.is_none() || !first_char.unwrap().is_alphabetic() {
            None
        } else {
            self.input = &self.input[1..];
            self.pos += 1;
            Some(first_char.unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokenize_single_digit_num() {
        let expr = "1";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.peek().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.peek().unwrap().num.unwrap(), 1);
        token_list.next();
        assert_eq!(token_list.peek().is_none(), true);
    }

    #[test]
    fn tokenize_multiple_digit_num() {
        let expr = "1234";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.peek().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.peek().unwrap().num.unwrap(), 1234);
        token_list.next();
        assert_eq!(token_list.peek().is_none(), true);
    }

    #[test]
    fn tokenize_operators() {
        let expr = "+-*/";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Plus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Minus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Mul);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Div);
    }

    #[test]
    fn tokenize_parens() {
        let expr = "()";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::LParen);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::RParen);
    }

    #[test]
    fn skip_whitespaces() {
        let expr = "5 + 20 - 4";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Plus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Minus);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
    }

    #[test]
    fn tokenize_multi_length_operators() {
        let expr = "<=>===!=<>";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(
            token_list.next().unwrap().kind,
            super::TokenKind::LessThanOrEqual
        );
        assert_eq!(
            token_list.next().unwrap().kind,
            super::TokenKind::GreaterThanOrEqual
        );
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Equal);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::NotEqual);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::LessThan);
        assert_eq!(
            token_list.next().unwrap().kind,
            super::TokenKind::GreaterThan
        );
    }

    #[test]
    fn tokenize_single_char_ident() {
        let expr = "a b";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Ident);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Ident);
    }

    #[test]
    fn tokenize_semicolon() {
        let expr = ";";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Semicolon);
    }

    #[test]
    fn tokenize_equal_assign() {
        let expr = "===";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Equal);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Assign);
    }

    #[test]
    fn tokenize_program() {
        let expr = "a = 42; a;";
        let mut token_list = super::Tokenizer::new(expr).tokenize();
        // a = 42;
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Ident);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Assign);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Num);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Semicolon);
        // a;
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Ident);
        assert_eq!(token_list.next().unwrap().kind, super::TokenKind::Semicolon);
    }
}
