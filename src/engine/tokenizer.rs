use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    Reference { name: &'a str, span: Range<usize> },
}

pub fn tokenize(input: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'#' {
            if start < i {
                tokens.push(Token::Text(&input[start..i]));
            }

            let begin = i;
            i += 1;

            if i >= bytes.len() || !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
                tokens.push(Token::Text(&input[begin..begin + 1]));
                start = begin + 1;
                continue;
            }

            let name_start = i;

            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }

            let name = &input[name_start..i];

            tokens.push(Token::Reference {
                name,
                span: begin..i,
            });

            start = i;
        } else {
            i += 1;
        }
    }

    if start < input.len() {
        tokens.push(Token::Text(&input[start..]));
    }

    tokens
}
