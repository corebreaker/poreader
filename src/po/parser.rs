use super::{line::PoLine, reader::PoReader, unescape::Unescaper};
use crate::error::Error;
use regex::Regex;
use std::{io::Read, collections::HashMap};

pub struct PoParser {
    map_re: Regex,
    map_check_re: Regex,
    message_re: Regex,
    comment_re: Regex,
    unescaper: Unescaper,
}

impl PoParser {
    pub fn new() -> PoParser {
        PoParser {
            map_re: Regex::new(r"(\S+?)\s*=\s*(.*?)\s*;").unwrap(),
            map_check_re: Regex::new(r"^\s*(\S+?\s*=\s*.*?\s*;\s*)*$").unwrap(),
            message_re: Regex::new(
                r#"^\s*(?:#(~?\|?))?\s*(msgctxt|msgid|msgid_plural|msgstr(?:\[(?:0|[1-9]\d*)\])?)?\s*"(.*)"\s*$"#
            ).unwrap(),
            comment_re: Regex::new(r#"^\s*#(.)?\s*(.*)$"#).unwrap(),
            unescaper: Unescaper::new(),
        }
    }

    pub fn parse<R: Read>(&self, reader: R) -> Result<PoReader<R>, Error> {
        PoReader::new(reader, self)
    }

    pub(crate) fn parse_map<'a>(&self, text: &'a str) -> Result<HashMap<&'a str, &'a str>, Error> {
        if self.map_check_re.is_match(text) {
            Ok(self.map_re.captures_iter(text)
                .filter_map(|c| c.get(1).and_then(|v1| c.get(2).map(|v2| (v1.as_str(), v2.as_str()))))
                .collect())
        } else {
            Err(Error::Unexpected(0, format!("Bad value list definition: `{}`", text)))
        }
    }

    pub(super) fn parse_line(&self, line: &str, n: usize) -> Result<PoLine, ()> {
        if !line.contains(|c: char| !c.is_whitespace()) {
            Ok(PoLine::Blank)
        } else if let Some(c) = self.message_re.captures(line) {
            let string = self.unescaper.unescape(c.get(3).map(|m| m.as_str()).unwrap_or_default());
            let flags = c.get(1).map(|x| x.as_str().to_string()).unwrap_or_default();

            Ok(match c.get(2) {
                None => PoLine::Continuation(n, flags, string),
                Some(m) => {
                    let tag = if flags.ends_with('|') {
                        String::from("|") + m.as_str()
                    } else {
                        m.as_str().to_string()
                    };

                    PoLine::Message(n, flags, tag, string)
                }
            })
        } else {
            self.comment_re.captures(line).map_or(Err(()), |c| Ok(PoLine::Comment(
                n,
                c.get(1).and_then(|m| m.as_str().chars().next()).unwrap_or(' '),
                c.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        source: &'static str,
        target: Result<PoLine, ()>,
    }

    impl TestCase {
        fn test(&self, parser: &PoParser) {
            assert_eq!(parser.parse_line(self.source, 123), self.target, "Error for source: `{}`", self.source);
        }
    }

    #[test]
    fn test_func_parse_map() {
        let parser = PoParser::new();

        match parser.parse_map("key=value") {
            Err(err) => assert_eq!(format!("{:?}", err), "Unexpected error: Bad value list definition: `key=value`"),
            v => panic!("Unexpected result: {:?}", v)
        }

        assert_eq!(parser.parse_map("     xyz=abc \t ; a=b;c=  d;   e  \t = f;"), Ok(vec![
            ("xyz", "abc"),
            ("a", "b"),
            ("c", "d"),
            ("e", "f"),
        ].into_iter().collect::<HashMap<_, _>>()));
    }

    #[test]
    fn test_func_parse_line() {
        let parser = PoParser::new();
        let cases = vec![
            TestCase { source: "---", target: Err(()) },
            TestCase { source: "\"--", target: Err(()) },
            TestCase { source: "msgid \"--", target: Err(()) },
            TestCase { source: "msgxx \"--\"", target: Err(()) },
            TestCase { source: "-# Something", target: Err(()) },
            TestCase { source: "", target: Ok(PoLine::Blank) },
            TestCase { source: "      ", target: Ok(PoLine::Blank) },
            TestCase { source: "   \t\r\n   ", target: Ok(PoLine::Blank) },
            TestCase {
                source: r#"msgid "hello\n\tworld""#,
                target: Ok(PoLine::Message(
                    123,
                    String::new(),
                    String::from("msgid"),
                    String::from("hello\n\tworld")
                )),
            },
            TestCase {
                source: r#"#| msgstr[3] "hello\n\tworld""#,
                target: Ok(PoLine::Message(
                    123,
                    String::from("|"),
                    String::from("|msgstr[3]"),
                    String::from("hello\n\tworld")
                )),
            },
            TestCase {
                source: r#""hello\n\tworld""#,
                target: Ok(PoLine::Continuation(
                    123,
                    String::new(),
                    String::from("hello\n\tworld")
                )),
            },
            TestCase {
                source: r#"#| "path: xx\\yy""#,
                target: Ok(PoLine::Continuation(
                    123,
                    String::from("|"),
                    String::from("path: xx\\yy")
                )),
            },
            TestCase {
                source: "#, My comment",
                target: Ok(PoLine::Comment(
                    123,
                    ',',
                    String::from("My comment")
                )),
            },
            TestCase {
                source: "# Another comment",
                target: Ok(PoLine::Comment(
                    123,
                    ' ',
                    String::from("Another comment")
                )),
            },
            TestCase {
                source: "#$ Special comment",
                target: Ok(PoLine::Comment(
                    123,
                    '$',
                    String::from("Special comment")
                )),
            },
        ];

        for case in cases {
            case.test(&parser);
        }
    }
}
