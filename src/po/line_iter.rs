use super::{line::PoLine, PoParser};
use crate::error::Error;
use std::io::{BufRead, BufReader, Lines, Read};

pub(super) struct LineIter<'p, R: Read> {
    n: usize,
    inner: Option<Lines<BufReader<R>>>,
    parser: &'p PoParser,
}

impl<'p, R: Read> LineIter<'p, R> {
    pub(super) fn new(r: R, parser: &'p PoParser) -> Self {
        Self {
            n: 1,
            inner: Some(BufReader::new(r).lines()),
            parser,
        }
    }
}

impl<'p, R: Read> Iterator for LineIter<'p, R> {
    type Item = Result<PoLine, Error>;

    fn next(&mut self) -> Option<Result<PoLine, Error>> {
        while let Some(reader) = self.inner.as_mut() {
            let n = self.n;
            let line = match reader.next() {
                Some(Ok(s)) => s,
                Some(Err(e)) => {
                    self.inner = None;

                    return Some(Err(Error::Io(n, e)));
                    // no-coverage:start
                }
                // no-coverage:stop
                None => {
                    return None;
                }
            };

            self.n += 1;

            match self.parser.parse_line(&line, n) {
                Ok(PoLine::Blank) => (),
                Ok(p) => return Some(Ok(p)),
                Err(()) => {
                    self.inner = None;

                    return Some(Err(Error::Parse(self.n, line, String::new())));
                }
            }
        }

        None
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let lines = "";
        let parser = PoParser::new();
        let mut iter = LineIter::new(lines.as_bytes(), &parser);

        if let Some(v) = iter.next() {
            panic!("Unexpected result for the first line: {:?}", v);
        }
    }

    #[test]
    fn blank() {
        let lines = "          \n\t\t   \n\n\n   \t\r\n";
        let parser = PoParser::new();
        let mut iter = LineIter::new(lines.as_bytes(), &parser);

        if let Some(v) = iter.next() {
            panic!("Unexpected result for the first line: {:?}", v);
        }
    }

    #[test]
    fn two_strict_lines() {
        let lines = "msgid \"Line 1\"\nmsgstr \"Line 2\"";
        let parser = PoParser::new();
        let mut iter = LineIter::new(lines.as_bytes(), &parser);

        match iter.next() {
            Some(Ok(PoLine::Message(line, flag, tag, string))) => {
                assert_eq!(line, 1);
                assert_eq!(flag, "");
                assert_eq!(tag, "msgid");
                assert_eq!(string, "Line 1");
            }
            v => panic!("Unexpected result for the first line: {:?}", v),
        }

        match iter.next() {
            Some(Ok(PoLine::Message(line, flag, tag, string))) => {
                assert_eq!(line, 2);
                assert_eq!(flag, "");
                assert_eq!(tag, "msgstr");
                assert_eq!(string, "Line 2");
            }
            v => panic!("Unexpected result for the second line: {:?}", v),
        }

        if let Some(v) = iter.next() {
            panic!("Unexpected result for the third line: {:?}", v);
        }
    }

    #[test]
    fn two_normal_lines() {
        let lines = r#"
            #~ msgid "Line 1"
            #~ msgstr "Line 2"
        "#;

        let parser = PoParser::new();
        let mut iter = LineIter::new(lines.as_bytes(), &parser);

        match iter.next() {
            Some(Ok(PoLine::Message(line, flag, tag, string))) => {
                assert_eq!(line, 2, "Bad line with flag={}, tag={}, string={}", flag, tag, string);
                assert_eq!(flag, "~");
                assert_eq!(tag, "msgid");
                assert_eq!(string, "Line 1");
            }
            v => panic!("Unexpected result for the first line: {:?}", v),
        }

        match iter.next() {
            Some(Ok(PoLine::Message(line, flag, tag, string))) => {
                assert_eq!(line, 3, "Bad line with flag={}, tag={}, string={}", flag, tag, string);
                assert_eq!(flag, "~");
                assert_eq!(tag, "msgstr");
                assert_eq!(string, "Line 2");
            }
            v => panic!("Unexpected result for the second line: {:?}", v),
        }

        if let Some(v) = iter.next() {
            panic!("Unexpected result for the third line: {:?}", v);
        }
    }

    #[test]
    fn with_parse_error() {
        let lines = r#"
            #: File:1
            msgid "Line 1"
            msgstr "Line 2

            # End
        "#;

        let parser = PoParser::new();
        let mut iter = LineIter::new(lines.as_bytes(), &parser);

        match iter.next() {
            Some(Ok(PoLine::Comment(line, kind, content))) => {
                assert_eq!(line, 2, "Bad line with kind={}, content={}", kind, content);
                assert_eq!(kind, ':');
                assert_eq!(content, "File:1");
            }
            v => panic!("Unexpected result for the first line: {:?}", v),
        }

        match iter.next() {
            Some(Ok(PoLine::Message(line, flag, tag, string))) => {
                assert_eq!(line, 3, "Bad line with flag={}, tag={}, string={}", flag, tag, string);
                assert_eq!(flag, "");
                assert_eq!(tag, "msgid");
                assert_eq!(string, "Line 1");
            }
            v => panic!("Unexpected result for the second line: {:?}", v),
        }

        match iter.next() {
            Some(Err(err)) => {
                assert_eq!(
                    format!("{:?}", err),
                    "Parse error at line 5, got ‘            msgstr \"Line 2’"
                );
            }
            v => panic!("Unexpected result for the third line: {:?}", v),
        }

        if let Some(v) = iter.next() {
            panic!("Unexpected result for the fourth line: {:?}", v);
        }
    }

    #[test]
    fn with_io_error() {
        let input = b"ABC\x32\x80\x32";

        let parser = PoParser::new();
        let mut iter = LineIter::new(&input[..], &parser);

        match iter.next() {
            Some(Err(_)) => {}
            v => {
                panic!("Error expected, got {v:?}");
            }
        }
    }
}
// no-coverage:stop
