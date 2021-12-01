use super::{line::PoLine, line_iter::LineIter};
use crate::{error::Error, unit::Unit};
use std::{io::Read, iter::Peekable};

#[inline]
fn fetch_next<R: Read>(reader: &mut Peekable<LineIter<R>>) -> Result<Option<PoLine>, Error> {
    if let Some(Ok(line)) = reader.peek() {
        return Ok(Some(line.clone()));
    }

    match reader.next() {
        Some(Err(err)) => Err(err),
        _ => Ok(None),
    }
}

pub(crate) trait Decoder {
    fn parse_msg(&mut self, tag: &str, unit: &Unit) -> Result<Option<String>, Error>;
    fn expected(&mut self, exp: &str) -> Result<(), Error>;
}

impl<'p, R: Read> Decoder for Peekable<LineIter<'p, R>> {
    fn parse_msg(&mut self, tag: &str, unit: &Unit) -> Result<Option<String>, Error> {
        let (prefix, mut string) = match fetch_next(self)? {
            Some(PoLine::Message(_, p, t, _)) if t == tag && p.starts_with('~') == unit.obsolete => {
                match self.next().unwrap().unwrap() {
                    PoLine::Message(_, p, _, s) => (p, s),
                    _ => {
                        unreachable!();
                    }
                }
            }
            _ => {
                return Ok(None);
            }
        };

        while let Some(PoLine::Continuation(_, ref p, _)) = fetch_next(self)? {
            if *p != prefix {
                break;
            }

            match self.next().unwrap().unwrap() {
                PoLine::Continuation(_, _, s) => {
                    string.push_str(&s);
                }
                _ => {
                    unreachable!();
                }
            }
        }

        Ok(Some(string))
    }

    fn expected(&mut self, exp: &str) -> Result<(), Error> {
        match self.peek() {
            None | Some(Ok(PoLine::Blank)) => Ok(()),
            Some(Err(_)) => {
                if let Some(Err(err)) = self.next() {
                    Err(err)
                } else {
                    unreachable!();
                }
            }
            Some(Ok(PoLine::Message(n, p, ..))) => Err(Error::Parse(*n, p.clone(), exp.to_string())),
            Some(Ok(PoLine::Continuation(n, ..))) => Err(Error::Parse(*n, String::from("\""), exp.to_string())),
            Some(Ok(PoLine::Comment(n, c, ..))) => Err(Error::Parse(*n, format!("#{}", c), exp.to_string())),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::PoParser;
    use super::*;
    use std::collections::{hash_map::Entry, HashMap};

    type Str = &'static str;

    pub(crate) enum TestAction<T> {
        ActOk(T),
        ActErr(Error),
        ActDelayed(Error),
    }

    impl TestAction<&str> {
        fn convert(self) -> TestAction<String> {
            match self {
                TestAction::ActOk(v) => TestAction::ActOk(v.to_string()),
                TestAction::ActErr(err) => TestAction::ActErr(err),
                TestAction::ActDelayed(err) => TestAction::ActDelayed(err),
            }
        }
    }

    pub(crate) struct TestDecoder {
        cnt: usize,
        command: String,
        message: String,
        log: Vec<String>,
        error: Option<Error>,
        values: Option<HashMap<String, TestAction<String>>>,
    }

    impl TestDecoder {
        pub(crate) fn new() -> TestDecoder {
            TestDecoder {
                cnt: 0,
                command: String::new(),
                message: String::from("message-0"),
                log: vec![],
                error: None,
                values: None,
            }
        }

        pub(crate) fn with_values<I: IntoIterator<Item = (Str, TestAction<Str>)>>(values: I) -> TestDecoder {
            let mut res = TestDecoder::new();

            res.push_values(values);
            res
        }

        pub(crate) fn set_message(&mut self, message: String) {
            self.message = message;
        }

        pub(crate) fn set_command(&mut self, command: String) {
            self.command = command;
        }

        pub(crate) fn push_values<I: IntoIterator<Item = (Str, TestAction<Str>)>>(&mut self, values: I) {
            match self.values.as_mut() {
                None => {
                    self.values
                        .replace(values.into_iter().map(|(k, v)| (k.to_string(), v.convert())).collect());
                }
                Some(self_values) => {
                    self_values.extend(values.into_iter().map(|(k, v)| (k.to_string(), v.convert())));
                }
            }
        }

        pub(crate) fn inc(&mut self) {
            self.cnt += 1;
            self.message = format!("message-{}", self.cnt);
        }

        pub(crate) fn log(&self) -> &Vec<String> {
            &self.log
        }

        pub(crate) fn set_error(&mut self, err: Error) {
            self.error.replace(err);
        }
    }

    impl Decoder for TestDecoder {
        fn parse_msg(&mut self, tag: &str, _unit: &Unit) -> Result<Option<String>, Error> {
            if let Some(err) = self.error.take() {
                return Err(err);
            }

            self.log.push(format!("Message: {}/{}", self.message, tag));

            match self.command.as_str() {
                "@DoError" => self.set_error(Error::Unexpected(210, String::from("From command `@DoError`"))),
                "@DoInc" => self.inc(),
                _ => (),
            }

            match self.values.as_mut() {
                None => Ok(Some(self.message.clone())),
                Some(map) => match map.entry(tag.to_string()) {
                    Entry::Vacant(_) => Ok(None),
                    Entry::Occupied(mut entry) => match entry.get_mut() {
                        TestAction::ActOk(v) => Ok(Some(v.clone())),
                        TestAction::ActErr(_) => Err(match entry.remove_entry().1 {
                            TestAction::ActErr(err) => err,
                            _ => unreachable!(),
                        }),
                        TestAction::ActDelayed(_) => Ok(match entry.remove_entry().1 {
                            TestAction::ActDelayed(err) => {
                                self.set_error(err);
                                None
                            }
                            _ => unreachable!(),
                        }),
                    },
                },
            }
        }

        fn expected(&mut self, exp: &str) -> Result<(), Error> {
            if exp == self.message {
                self.log.push(format!("Expected: {}", self.message));
                if let Some(err) = self.error.take() {
                    return Err(err);
                }
            }

            Ok(())
        }
    }

    #[test]
    fn test_func_fetch_next() {
        let parser = PoParser::new();
        let mut iter = LineIter::new("msgid \"line 1\"\nmsgstr \"line 2\"".as_bytes(), &parser).peekable();

        match fetch_next(&mut iter) {
            Ok(Some(line)) => match line {
                PoLine::Message(line, flag, tag, string) => {
                    assert_eq!(line, 1);
                    assert_eq!(flag, "");
                    assert_eq!(tag, "msgid");
                    assert_eq!(string, "line 1");
                }
                r => panic!("Unexpected line: {:?}", r),
            },
            r => panic!("Unexpected result: {:?}", r),
        }

        iter.next();
        match fetch_next(&mut iter) {
            Ok(Some(line)) => match line {
                PoLine::Message(line, flag, tag, string) => {
                    assert_eq!(line, 2);
                    assert_eq!(flag, "");
                    assert_eq!(tag, "msgstr");
                    assert_eq!(string, "line 2");
                }
                r => panic!("Unexpected line: {:?}", r),
            },
            r => panic!("Unexpected result: {:?}", r),
        }

        iter.next();
        match fetch_next(&mut iter) {
            Ok(None) => (),
            r => panic!("Unexpected result: {:?}", r),
        }

        let mut iter = LineIter::new("msgid \"line 1".as_bytes(), &parser).peekable();

        match fetch_next(&mut iter) {
            Err(err) => assert_eq!(
                format!("{:?}", err),
                String::from("Parse error at line 2, got ‘msgid \"line 1’")
            ),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_msg() {
        let parser = PoParser::new();

        {
            let text = "#~ msgid \"this\"\nmsgid \"that\"";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();
            let mut unit = Unit::default();

            assert_eq!(lines.parse_msg("---", &unit), Ok(None));
            assert_eq!(lines.parse_msg("msgid", &unit), Ok(None));

            unit = Unit::for_tests_incomplete();

            assert_eq!(lines.parse_msg("msgid", &unit), Ok(Some(String::from("this"))));

            assert_eq!(lines.parse_msg("---", &unit), Ok(None));
            assert_eq!(lines.parse_msg("msgid", &unit), Ok(None));
        }

        {
            let text = "msgid \"this\"\n\" is\"\n\" good\"";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();
            let unit = Unit::default();

            assert_eq!(lines.parse_msg("msgid", &unit), Ok(Some(String::from("this is good"))));
        }

        {
            let text = "msgid \"this";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();
            let unit = Unit::default();

            match lines.parse_msg("msgid", &unit) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 2, got ‘msgid \"this’"),
                v => panic!("Unexpected result for the first error: {:?}", v),
            }
        }

        {
            let text = "msgid \"this\"\n\" is bad";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();
            let unit = Unit::default();

            match lines.parse_msg("msgid", &unit) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 3, got ‘\" is bad’"),
                v => panic!("Unexpected result for the second error: {:?}", v),
            }
        }
    }

    #[test]
    fn test_func_expected() {
        let parser = PoParser::new();

        {
            let text = "   ";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();

            assert_eq!(lines.expected(""), Ok(()));

            lines.next();
            assert_eq!(lines.expected(""), Ok(()));
        }

        {
            let text = "---";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();

            match lines.expected("") {
                Err(err) => assert_eq!(format!("{:?}", err), String::from("Parse error at line 2, got ‘---’")),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let text = "# this is a test\nmsgid \"hello,\"\n\"it's me\"";
            let mut lines = LineIter::new(text.as_bytes(), &parser).peekable();

            match lines.expected("here-1") {
                Err(err) => {
                    let msg = String::from("Parse error at line 1 expected ‘here-1’, got ‘# ’");

                    assert_eq!(format!("{:?}", err), msg);
                }
                r => panic!("Unexpected result: {:?}", r),
            }

            lines.next();
            match lines.expected("here-2") {
                Err(err) => {
                    let msg = String::from("Parse error at line 2 expected ‘here-2’");

                    assert_eq!(format!("{:?}", err), msg);
                }
                r => panic!("Unexpected result: {:?}", r),
            }

            lines.next();
            match lines.expected("here-3") {
                Err(err) => {
                    let msg = String::from("Parse error at line 3 expected ‘here-3’, got ‘\"’");

                    assert_eq!(format!("{:?}", err), msg);
                }
                r => panic!("Unexpected result: {:?}", r),
            }
        }
    }
}
