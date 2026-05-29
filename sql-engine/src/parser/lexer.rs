#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Create,
    Table,
    Drop,
    Delete,
    Update,
    Set,
    And,
    Or,
    Not,
    Identifier,
    IntLiteral,
    StringLiteral,
    FloatLiteral,
    BoolLiteral,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Comma,
    LParen,
    RParen,
    Star,
    Semicolon,
    Eof,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        let ch = chars[pos];

        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch == '-' && pos + 1 < chars.len() && chars[pos + 1] == '-' {
            while pos < chars.len() && chars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        if ch == '=' {
            tokens.push(Token { kind: TokenKind::Eq, lexeme: "=".into() });
        } else if ch == '!' && pos + 1 < chars.len() && chars[pos + 1] == '=' {
            tokens.push(Token { kind: TokenKind::Neq, lexeme: "!=".into() });
            pos += 1;
        } else if ch == '<' && pos + 1 < chars.len() && chars[pos + 1] == '=' {
            tokens.push(Token { kind: TokenKind::Lte, lexeme: "<=".into() });
            pos += 1;
        } else if ch == '>' && pos + 1 < chars.len() && chars[pos + 1] == '=' {
            tokens.push(Token { kind: TokenKind::Gte, lexeme: ">=".into() });
            pos += 1;
        } else if ch == '<' {
            tokens.push(Token { kind: TokenKind::Lt, lexeme: "<".into() });
        } else if ch == '>' {
            tokens.push(Token { kind: TokenKind::Gt, lexeme: ">".into() });
        } else if ch == ',' {
            tokens.push(Token { kind: TokenKind::Comma, lexeme: ",".into() });
        } else if ch == '(' {
            tokens.push(Token { kind: TokenKind::LParen, lexeme: "(".into() });
        } else if ch == ')' {
            tokens.push(Token { kind: TokenKind::RParen, lexeme: ")".into() });
        } else if ch == '*' {
            tokens.push(Token { kind: TokenKind::Star, lexeme: "*".into() });
        } else if ch == ';' {
            tokens.push(Token { kind: TokenKind::Semicolon, lexeme: ";".into() });
        } else if ch == '\'' || ch == '"' {
            let quote = ch;
            let mut s = String::new();
            pos += 1;
            while pos < chars.len() && chars[pos] != quote {
                s.push(chars[pos]);
                pos += 1;
            }
            tokens.push(Token { kind: TokenKind::StringLiteral, lexeme: s });
        } else if ch.is_alphabetic() || ch == '_' {
            let mut s = String::new();
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                s.push(chars[pos].to_ascii_lowercase());
                pos += 1;
            }
            pos -= 1;

            let kind = match s.as_str() {
                "select" => TokenKind::Select,
                "from" => TokenKind::From,
                "where" => TokenKind::Where,
                "insert" => TokenKind::Insert,
                "into" => TokenKind::Into,
                "values" => TokenKind::Values,
                "create" => TokenKind::Create,
                "table" => TokenKind::Table,
                "drop" => TokenKind::Drop,
                "delete" => TokenKind::Delete,
                "update" => TokenKind::Update,
                "set" => TokenKind::Set,
                "and" => TokenKind::And,
                "or" => TokenKind::Or,
                "not" => TokenKind::Not,
                "true" => TokenKind::BoolLiteral,
                "false" => TokenKind::BoolLiteral,
                _ => TokenKind::Identifier,
            };
            tokens.push(Token { kind, lexeme: s });
        } else if ch.is_numeric() {
            let mut s = String::new();
            let mut is_float = false;
            while pos < chars.len() && (chars[pos].is_numeric() || chars[pos] == '.') {
                if chars[pos] == '.' {
                    is_float = true;
                }
                s.push(chars[pos]);
                pos += 1;
            }
            pos -= 1;
            tokens.push(Token {
                kind: if is_float { TokenKind::FloatLiteral } else { TokenKind::IntLiteral },
                lexeme: s,
            });
        }
        pos += 1;
    }
    tokens.push(Token { kind: TokenKind::Eof, lexeme: "".into() });
    tokens
}
