use std::borrow::Cow;
use std::fmt::{self, Write};

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
    index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    Atom(&'a str),
    Special([u8; 2]),
    String(Cow<'a, str>),
}

const SPECIAL: &[u8] = b"[]{}()'`~^@";

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Atom(a) => f.write_str(a),
            Token::Special([b, b'\0']) => f.write_char(*b as char),
            Token::Special([b1, b2]) => {
                f.write_char(*b1 as char)?;
                f.write_char(*b2 as char)
            }
            Token::String(s) => fmt::Debug::fmt(s, f),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.curr()? {
                b'~' => {
                    self.eat(1);
                    if self.curr() == Some(b'@') {
                        self.eat(1);
                        break Some(Token::Special([b'~', b'@']));
                    } else {
                        break Some(Token::Special([b'~', b'\0']));
                    }
                }
                b if SPECIAL.contains(&b) => {
                    self.eat(1);
                    break Some(Token::Special([b, b'\0']));
                }
                b'"' => break self.eat_string().map(Token::String),
                b';' => self.eat_comment(),
                b',' => self.eat(1),
                b if b.is_ascii_whitespace() => self.eat(1),
                _ => break self.eat_atom().map(Token::Atom),
            }
        }
    }
}

impl<'a> Lexer<'a> {
    fn curr(&self) -> Option<u8> {
        self.source.as_bytes().get(self.index).copied()
    }

    fn eat(&mut self, how_many: usize) {
        self.index += how_many;
    }

    fn eat_comment(&mut self) {
        self.index = self.source[self.index..]
            .find('\n')
            .map(|i| i + self.index)
            .unwrap_or(self.source.len());
    }

    fn eat_string(&mut self) -> Option<Cow<'a, str>> {
        self.eat(1); // `"`
        let source = &self.source[self.index..];

        let i = source.find(&['\\', '"'])?;
        if source.as_bytes()[i] == b'\\' {
            self.eat_escaped_string(i).map(Cow::Owned)
        } else {
            let s = &source[..i];
            self.index += i + 1;
            Some(Cow::Borrowed(s))
        }
    }

    fn eat_escaped_string(&mut self, first_escape_index: usize) -> Option<String> {
        let source = &self.source[self.index..];
        let mut escaped = source[..first_escape_index].to_owned();
        let mut index = first_escape_index + 1;

        loop {
            match source.as_bytes().get(index)? {
                b'n' => escaped.push('\n'),
                b'\\' => escaped.push('\\'),
                b'"' => escaped.push('\"'),
                _ => return None,
            }
            index += 1;

            let subsource = &source[index..];

            let i = subsource.find(&['\\', '"'])?;
            escaped.push_str(&subsource[..i]);
            index += i + 1;
            if subsource.as_bytes()[i] == b'\\' {
                continue;
            } else {
                self.index += index;
                break;
            }
        }

        Some(escaped)
    }

    fn eat_atom(&mut self) -> Option<&'a str> {
        let start_index = self.index;
        loop {
            match self.curr() {
                Some(b) if SPECIAL.contains(&b) => break,
                Some(b';' | b',') => break,
                Some(b) if b.is_ascii_whitespace() => break,
                Some(_) => {
                    self.eat(1);
                    continue;
                }
                None => break,
            }
        }
        Some(&self.source[start_index..self.index])
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{Lexer, Token};

    #[test]
    fn special() {
        let chars = "[]{}()'`~^@";
        for i in 0..chars.len() {
            let mut lex = Lexer::new(&chars[i..i + 1]);
            assert_eq!(
                lex.next(),
                Some(Token::Special([chars.as_bytes()[i], b'\0']))
            );
            assert!(lex.next().is_none());
        }
    }

    #[test]
    fn combined_special() {
        use Token::{Atom as A, Special as S, String as Str};

        let cases = [
            ("~1", &[S([b'~', b'\0']), A("1")][..]),
            (r#"~"abc""#, &[S([b'~', b'\0']), Str("abc".into())][..]),
            ("~^1", &[S([b'~', b'\0']), S([b'^', b'\0']), A("1")][..]),
            ("@1", &[S([b'@', b'\0']), A("1")][..]),
            ("~@1", &[S([b'~', b'@']), A("1")][..]),
            ("~ @1", &[S([b'~', b'\0']), S([b'@', b'\0']), A("1")][..]),
            ("~,@,1", &[S([b'~', b'\0']), S([b'@', b'\0']), A("1")][..]),
        ];

        for (input, expected) in cases {
            let lex = Lexer::new(input);
            let lexed: Vec<_> = lex.collect();
            assert_eq!(lexed.as_slice(), expected);
        }
    }

    #[test]
    fn atom() {
        let cases = [
            "1",
            "abc",
            "1.0",
            "hello<>world",
            "a-b_c%d#e!f*g",
            "   a-b_c%d#e!f*g",
            "a-b_c%d#e!f*g   ",
            "   a-b_c%d#e!f*g   ",
        ];

        for case in cases {
            let mut lex = Lexer::new(case);
            assert_eq!(lex.next(), Some(Token::Atom(case.trim())));
            assert!(lex.next().is_none());
        }
    }

    #[test]
    fn string() {
        let cases = [
            (r##""""##, ""),
            (r##""abc""##, "abc"),
            (r##""abc def""##, "abc def"),
            (
                r##""abc123!@#$%^&*)(_+-=}{][|;':/?.>,<""##,
                "abc123!@#$%^&*)(_+-=}{][|;':/?.>,<",
            ),
            (r##""abc\ndef""##, "abc\ndef"),
            (r##""abc\\def""##, "abc\\def"),
            (r##""abc\"def""##, "abc\"def"),
            (r##""abc\ndef\"""##, "abc\ndef\""),
        ];

        for (input, expected) in cases {
            let mut lex = Lexer::new(input);
            assert_eq!(lex.next(), Some(Token::String(Cow::Borrowed(expected))));
            assert!(lex.next().is_none());
        }
    }

    #[test]
    fn multiple() {
        use Token::{Atom as A, Special as S};
        let ss = |c: char| S([c as u8, b'\0']);
        let cases = [
            ("(123 456)", &[ss('('), A("123"), A("456"), ss(')')][..]),
            (
                "( 123 456 789 )",
                &[ss('('), A("123"), A("456"), A("789"), ss(')')],
            ),
            ("(a,b,c)", &[ss('('), A("a"), A("b"), A("c"), ss(')')]),
            (
                "( 123 456 789 ) ; (hmm 123)",
                &[ss('('), A("123"), A("456"), A("789"), ss(')')],
            ),
            (
                "( + 2 (* 3 4) )",
                &[
                    ss('('),
                    A("+"),
                    A("2"),
                    ss('('),
                    A("*"),
                    A("3"),
                    A("4"),
                    ss(')'),
                    ss(')'),
                ],
            ),
            (
                ",(,+,2,(,*,3,4,),),",
                &[
                    ss('('),
                    A("+"),
                    A("2"),
                    ss('('),
                    A("*"),
                    A("3"),
                    A("4"),
                    ss(')'),
                    ss(')'),
                ],
            ),
        ];

        for (input, expected) in cases {
            let lex = Lexer::new(input);
            let lexed: Vec<_> = lex.collect();
            assert_eq!(lexed.as_slice(), expected);
        }
    }
}
