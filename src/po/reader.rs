use super::{line::PoLine, line_iter::LineIter, parser::PoParser, MessageExtractor as Extractor};
use crate::{
    comment::Comment, error::Error, note::Note, plural::PluralForms, unit::Unit, CatalogueReader, Origin, State,
};

use locale_config::LanguageRange;
use std::{
    collections::HashMap,
    io::Read,
    iter::Peekable,
    mem::{replace, swap},
    rc::Rc,
};

/// Object for reading PO streams
///
/// An iterator is implemented for reading each unit of translation in the PO stream.
pub struct PoReader<'p, R: Read> {
    lines: Peekable<LineIter<'p, R>>,
    next_unit: Option<Result<Unit, Error>>,
    header_notes: Vec<Note>,
    header_comments: Vec<Comment>,
    header_properties: HashMap<String, String>,
    target_language: LanguageRange<'static>,
    plural_forms: Option<Rc<PluralForms>>,
}

impl<'p, R: Read> PoReader<'p, R> {
    pub(super) fn new(reader: R, parser: &'p PoParser) -> Result<PoReader<'p, R>, Error> {
        let mut res = PoReader {
            lines: LineIter::new(reader, parser).peekable(),
            next_unit: None,
            header_notes: vec![],
            header_comments: vec![],
            header_properties: HashMap::new(),
            target_language: LanguageRange::invariant(),
            plural_forms: None,
        };

        let (next_unit, has_header) = match res.next_unit(true) {
            Some(Err(err)) => {
                return Err(err);
            }
            Some(Ok(u)) => {
                let has_header = u.message().is_empty();

                (Some(Ok(u)), has_header)
            }
            u => (u, false),
        };

        res.next_unit = next_unit;
        if has_header {
            res.parse_po_header(parser)?;
            res.next_unit = res.next_unit(false);
        }

        Ok(res)
    }

    fn read_line(&mut self) -> Result<Option<(usize, bool)>, Error> {
        match self.lines.peek() {
            // end if no unit (possibly after comments)
            None => Ok(None),

            // error
            Some(Err(_)) => {
                if let Some(Err(err)) = replace(&mut self.next_unit, None) {
                    Err(err)
                } else if let Some(Err(err)) = self.lines.next() {
                    Err(err)
                } else {
                    unreachable!();
                }
            }

            // detect obsolete
            Some(Ok(PoLine::Message(line, p, ..))) if p.starts_with('~') => Ok(Some((*line, true))),

            // normal line
            Some(Ok(v)) => Ok(Some((v.line(), false))),
        }
    }

    fn parse_comments(&mut self, unit: &mut Unit) -> Result<(), Error> {
        while let Some(Ok(PoLine::Comment(..))) = self.lines.peek() {
            match self.lines.next() {
                Some(Ok(PoLine::Comment(_, ',', s))) => {
                    for flag in s.split(',').map(str::trim) {
                        unit.flags.insert(flag.to_string());

                        match flag {
                            "fuzzy" => unit.state = State::NeedsWork,
                            _ => (), // TODO: Implement other flags (do we need any?)
                        }
                    }
                }
                Some(Ok(PoLine::Comment(_, ':', s))) => {
                    unit.locations
                        .extend(s.split(char::is_whitespace).filter(|x| !x.is_empty()).map(From::from));
                }
                Some(Ok(PoLine::Comment(_, '.', value))) => {
                    unit.notes.push(Note::new(Origin::Developer, value));
                }
                Some(Ok(PoLine::Comment(_, ' ', value))) => {
                    unit.notes.push(Note::new(Origin::Translator, value));
                }
                Some(Ok(PoLine::Comment(_, kind, content))) => {
                    unit.comments.push(Comment::new(kind, content));
                }
                _ => unreachable!(), // we *know* it is a Some(Ok(Comment))
            }
        }

        if let Some(Err(_)) = self.lines.peek() {
            if let Some(Err(err)) = self.lines.next() {
                Err(err)
            } else {
                unreachable!();
            }
        } else {
            Ok(())
        }
    }

    fn parse_unit(&mut self, unit: Unit, first: bool) -> Result<Option<Unit>, Error> {
        let plural_forms = self.plural_forms.as_ref().map(Rc::clone);
        let params = Extractor::new(unit, &mut self.lines, plural_forms);

        params.parse_message_fields(first)
    }

    fn read_unit(&mut self, first: bool) -> Result<Option<Unit>, Error> {
        let mut unit = Unit::default();

        self.parse_comments(&mut unit)?;

        let line = match self.read_line()? {
            None => {
                return Ok(None);
            }
            Some((line, is_obsolete)) => {
                unit.obsolete = is_obsolete;
                line
            }
        };

        unit = match self.parse_unit(unit, first)? {
            Some(unit) => unit,
            None => {
                return Ok(None);
            }
        };

        if (!first) && unit.message.is_empty() {
            Err(Error::Unexpected(line, String::from("Source should not be empty")))
        } else {
            if unit.state == State::Empty && !unit.message.is_blank() {
                // translation is non-empty and state was not set yet, then it is final
                unit.state = State::Final;
            }

            Ok(Some(unit))
        }
    }

    fn next_unit(&mut self, first: bool) -> Option<Result<Unit, Error>> {
        match self.read_unit(first) {
            Ok(None) => None,
            Ok(Some(u)) => Some(Ok(u)),
            Err(e) => Some(Err(e)),
        }
    }

    fn parse_po_header(&mut self, parser: &PoParser) -> Result<(), Error> {
        if let Some(Ok(ref u)) = self.next_unit {
            for line in u.message.get_text().split('\n') {
                if let Some(n) = line.find(':') {
                    let key = line[..n].trim();
                    let val = line[(n + 1)..].trim();

                    self.header_properties.insert(key.to_owned(), val.to_owned());
                }
            }

            self.header_notes.extend_from_slice(&u.notes);
            self.header_comments.extend_from_slice(&u.comments);

            if let Some(lang) = self.header_properties.get("Language") {
                self.target_language = LanguageRange::new(lang)
                    .map(LanguageRange::into_static)
                    .or_else(|_| LanguageRange::from_unix(lang))
                    .unwrap_or_else(|_| LanguageRange::invariant());
            }

            if let Some(forms) = self.header_properties.get("Plural-Forms") {
                if !forms.is_empty() {
                    self.plural_forms.replace(Rc::new(PluralForms::parse(forms, parser)?));
                }
            }
        }

        Ok(())
    }
}

impl<'p, R: Read> Iterator for PoReader<'p, R> {
    type Item = Result<Unit, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_unit {
            None => None,
            Some(Err(_)) => replace(&mut self.next_unit, None),
            _ => {
                let mut res = self.next_unit(false);

                swap(&mut res, &mut self.next_unit);
                res
            }
        }
    }
}

impl<'p, R: Read> CatalogueReader for PoReader<'p, R> {
    fn target_language(&self) -> &LanguageRange<'static> {
        &self.target_language
    }

    fn header_notes(&self) -> &Vec<Note> {
        &self.header_notes
    }

    fn header_comments(&self) -> &Vec<Comment> {
        &self.header_comments
    }

    fn header_properties(&self) -> &HashMap<String, String> {
        &self.header_properties
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use std::collections::HashSet;

    fn make_source() -> &'static str {
        "\
            #, flag1, flag2, fuzzy\n\
            #$ Any comment 1\n\
            #& Any comment 2\n\
            # Any note\n\
            #: File1:1 File1:2\n\
            #: File2:1 File2:2\n\
            msgctxt \"Any context\"\n\
            msgid \"\"\n\
            msgstr \"\"\n\
            \"Any-Header: Value\\n\"\n\
            \"Language: fr\\n\"\n\
            \n\
            msgid \"Hello, world !\"\n\
            msgstr \"Salut, tout le monde !\"\
        "
    }

    fn make_reader<R: Read>(reader: R, parser: &PoParser) -> PoReader<R> {
        let mut unit = Unit::default();

        unit.message = Message::Simple {
            id: String::new(),
            text: Some(String::from(
                "\
                    Header-1: Value1\n\
                    Language: en\n\
                    Plural-Forms: nplurals=2; plural=(n > 1);\n\
                    Header-2: Value2\
                ",
            )),
        };

        PoReader {
            lines: LineIter::new(reader, parser).peekable(),
            next_unit: Some(Ok(unit)),
            header_notes: vec![
                Note::new(Origin::Translator, String::from("You")),
                Note::new(Origin::Developer, String::from("Me")),
            ],
            header_comments: vec![
                Comment::new('+', String::from("Comment 1")),
                Comment::new('=', String::from("Comment 2")),
            ],
            header_properties: HashMap::new(),
            target_language: LanguageRange::invariant(),
            plural_forms: None,
        }
    }

    #[test]
    fn test_func_parse_po_header_with_error() {
        let mut unit = Unit::default();

        unit.message = Message::Simple {
            id: String::new(),
            text: Some(String::from(r"Plural-Forms: plural=1+;")),
        };

        let source = "";
        let parser = PoParser::new();
        let mut reader = make_reader(source.as_bytes(), &parser);

        reader.next_unit.replace(Ok(unit));
        match reader.parse_po_header(&parser) {
            Err(err) => assert_eq!(
                format!("{:?}", err),
                r##"Error in plurals forms: Unrecognized EOF found at 2
Expected one of "(", "-", "n" or r#"[0-9]+"#"##,
            ),
            Ok(_) => panic!(
                "Unexpected result: forms={:?}, notes={:?}, headers={:?}, next={:?}",
                reader.plural_forms, reader.header_notes, reader.header_properties, reader.next_unit,
            ),
        }
    }

    #[test]
    fn test_func_parse_po_header_with_bad_language() {
        let fallback = LanguageRange::invariant();

        let source = "";
        let parser = PoParser::new();
        let mut reader = make_reader(source.as_bytes(), &parser);

        let mut unit = Unit::default();

        unit.message = Message::Simple {
            id: String::new(),
            text: Some(String::from(r"Language: fx42")),
        };

        reader.next_unit.replace(Ok(unit));

        match reader.parse_po_header(&parser) {
            Ok(()) => assert_eq!(reader.target_language(), &fallback),
            Err(err) => panic!(
                "Unexpected error: {:?}\nforms={:?}, notes={:?}, headers={:?}, next={:?}",
                err, reader.plural_forms, reader.header_notes, reader.header_properties, reader.next_unit,
            ),
        }
    }

    #[test]
    fn test_func_parse_po_header_normal() {
        let source = "";
        let parser = PoParser::new();
        let mut reader = make_reader(source.as_bytes(), &parser);

        match reader.parse_po_header(&parser) {
            Ok(()) => {
                assert_eq!(
                    reader.header_notes,
                    vec![
                        Note::new(Origin::Translator, String::from("You")),
                        Note::new(Origin::Developer, String::from("Me")),
                    ]
                );

                let definition = "nplurals=2; plural=(n > 1);";

                assert_eq!(
                    reader.header_properties,
                    vec![
                        ("Header-1", "Value1"),
                        ("Header-2", "Value2"),
                        ("Language", "en"),
                        ("Plural-Forms", definition),
                    ]
                    .into_iter()
                    .map(|(k, v)| (String::from(k), String::from(v)))
                    .collect::<HashMap<_, _>>()
                );

                assert_eq!(
                    reader.header_comments,
                    vec![
                        Comment::new('+', String::from("Comment 1")),
                        Comment::new('=', String::from("Comment 2")),
                    ]
                );

                assert_eq!(reader.target_language.as_ref(), "en");

                if let Some(forms) = reader.plural_forms {
                    assert_eq!(forms.get_formula(), "(n > 1)");
                    assert_eq!(forms.get_definition(), definition);
                    assert_eq!(forms.get_count(), 2);
                } else {
                    panic!("Unexpected None for forms");
                }
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_func_read_line() {
        let parser = PoParser::new();

        {
            let source = "#~ msgid \"my-id-1\"\nmsgid \"my-id-2\"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_line() {
                Ok(Some((line, result))) => {
                    assert!(result, "Result should be true");
                    assert_eq!(line, 1);
                }
                v => panic!("Unexpected result for first line: {:?}", v),
            }

            if let None = reader.lines.next() {
                panic!("Unexpected None as first result of next");
            }

            match reader.read_line() {
                Ok(Some((line, result))) => {
                    assert!(!result, "Result should be false");
                    assert_eq!(line, 2);
                }
                v => panic!("Unexpected result for second line: {:?}", v),
            }

            if let None = reader.lines.next() {
                panic!("Unexpected None as second result of next");
            }

            match reader.read_line() {
                Ok(None) => (),
                v => panic!("Unexpected result for the end: {:?}", v),
            }
        }

        {
            let source = "msgid \"my-error";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_line() {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 2, got ‘msgid \"my-error’"),
                v => panic!("Unexpected result for the first error case: {:?}", v),
            }
        }

        {
            let source = "msgid \"my-error";
            let mut reader = make_reader(source.as_bytes(), &parser);

            reader.next_unit = Some(Err(Error::Unexpected(123, String::from("An error"))));
            match reader.read_line() {
                Err(err) => assert_eq!(format!("{:?}", err), "Unexpected error at line 123: An error"),
                v => panic!("Unexpected result for the second error case: {:?}", v),
            }
        }
    }

    #[test]
    fn test_func_parse_comments() {
        let parser = PoParser::new();

        {
            let source = "msgid \"my-error";
            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            match reader.parse_comments(&mut unit) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 2, got ‘msgid \"my-error’"),
                v => panic!("Unexpected result for the first error line: {:?}", v),
            }

            match reader.parse_comments(&mut unit) {
                Ok(()) => (),
                v => panic!("Unexpected result for the second error line: {:?}", v),
            }
        }

        {
            let source = "msgid \"my-id\"";
            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            match reader.parse_comments(&mut unit) {
                Ok(()) => (),
                v => panic!("Unexpected result for single line: {:?}", v),
            }
        }

        {
            let source = "# simple comment\nmsgid \"my-error";
            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            match reader.parse_comments(&mut unit) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 3, got ‘msgid \"my-error’"),
                v => panic!("Unexpected result for the third error line: {:?}", v),
            }

            match reader.parse_comments(&mut unit) {
                Ok(()) => assert_eq!(
                    unit.notes,
                    [Note::new(Origin::Translator, String::from("simple comment"))]
                ),
                v => panic!("Unexpected result for the fourth error line: {:?}", v),
            }
        }

        {
            let source = "#$ any comment\n# simple note";
            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            match reader.parse_comments(&mut unit) {
                Ok(()) => {
                    assert_eq!(unit.notes, [Note::new(Origin::Translator, String::from("simple note"))]);
                    assert_eq!(unit.comments, [Comment::new('$', String::from("any comment"))]);
                }
                Err(err) => panic!(
                    "Unexpected error: {:?}\nnotes={:?}, locations={:?}, flags={:?}",
                    err, unit.notes, unit.locations, unit.flags,
                ),
            }
        }

        {
            let source = "\
                #  translator comment\n\
                #. developer comment\n\
                #: Location:1 Location:2\n\
                #: Location:3 Location:4\n\
                #, flag1, flag2\n\
                #, flag3, fuzzy, flag4\n\
            ";

            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            match reader.parse_comments(&mut unit) {
                Ok(()) => {
                    assert_eq!(
                        unit.notes,
                        [
                            Note::new(Origin::Translator, String::from("translator comment")),
                            Note::new(Origin::Developer, String::from("developer comment")),
                        ]
                    );

                    assert_eq!(
                        unit.locations,
                        (1..=4).map(|i| format!("Location:{}", i)).collect::<Vec<_>>(),
                    );

                    assert_eq!(
                        unit.flags,
                        (1..=4)
                            .map(|i| format!("flag{}", i))
                            .chain(vec![String::from("fuzzy")])
                            .collect::<HashSet<_>>()
                    );

                    assert_eq!(unit.state, State::NeedsWork);
                }
                v => panic!("Unexpected result for single line: {:?}", v),
            }
        }
    }

    #[test]
    fn test_func_parse_unit() {
        let parser = PoParser::new();

        {
            let source = "msgid \"\"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.parse_unit(Unit::default(), false) {
                Ok(None) => (),
                v => panic!("Unexpected result for empty `msgid`: {:?}", v),
            }
        }

        {
            let source = "msgid \"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.parse_unit(Unit::default(), false) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 2, got ‘msgid \"’"),
                Ok(v) => panic!("Unexpected result for bad `msgid`: {:?}", v),
            }
        }

        {
            let source = make_source();
            let mut reader = make_reader(source.as_bytes(), &parser);
            let mut unit = Unit::default();

            reader
                .parse_comments(&mut unit)
                .expect("Comments should be parsed before tests");
            match reader.parse_unit(unit, true) {
                Ok(Some(unit)) => {
                    assert!(unit.message.is_empty(), "Message should be empty");
                    assert!(unit.prev_message.is_empty(), "Previous message should be empty");

                    assert_eq!(unit.message.get_text(), "Any-Header: Value\nLanguage: fr\n");

                    assert_eq!(unit.context(), Some("Any context"));
                    assert_eq!(unit.prev_context(), None);
                }
                v => panic!("Unexpected result for empty message (header): {:?}", v),
            }
        }
    }

    #[test]
    fn test_func_read_unit_normal() {
        let source = make_source();
        let parser = PoParser::new();
        let mut reader = make_reader(source.as_bytes(), &parser);

        match reader.read_unit(true) {
            Ok(Some(unit)) => {
                assert!(unit.message.is_empty(), "Message should be empty");
                assert!(unit.prev_message.is_empty(), "Previous message should be empty");
                assert!(!unit.obsolete, "This should not be obsolete");

                assert_eq!(
                    unit.comments,
                    vec![
                        Comment::new('$', String::from("Any comment 1")),
                        Comment::new('&', String::from("Any comment 2")),
                    ]
                );

                assert_eq!(
                    unit.locations,
                    ["File1:1", "File1:2", "File2:1", "File2:2",]
                        .into_iter()
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                );

                assert_eq!(
                    unit.flags,
                    ["flag1", "flag2", "fuzzy"].into_iter().map(str::to_string).collect()
                );
                assert_eq!(
                    unit.notes,
                    vec![Note::new(Origin::Translator, String::from("Any note"))]
                );

                assert_eq!(unit.state, State::NeedsWork);
                assert_eq!(unit.message.get_text(), "Any-Header: Value\nLanguage: fr\n");

                assert_eq!(unit.context(), Some("Any context"));
                assert_eq!(unit.prev_context(), None);
            }
            v => panic!("Unexpected result for empty message (header): {:?}", v),
        }

        match reader.read_unit(false) {
            Ok(Some(unit)) => {
                assert_eq!(unit.state, State::Final);
                assert_eq!(unit.message.get_id(), "Hello, world !");
                assert_eq!(unit.message.get_text(), "Salut, tout le monde !");
            }
            v => panic!("Unexpected result for empty message (header): {:?}", v),
        }
    }

    #[test]
    fn test_func_read_unit_with_exceptions() {
        let parser = PoParser::new();

        {
            let source = "msgid \"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_unit(false) {
                Err(err) => assert_eq!(format!("{:?}", err), "Parse error at line 2, got ‘msgid \"’"),
                Ok(r) => panic!("Unexpected result for the error test on parse comment: {:?}", r),
            }
        }

        {
            let source = "";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_unit(false) {
                Ok(None) => (),
                r => panic!("Unexpected result for the test on end of stream: {:?}", r),
            }
        }

        {
            let source = "#~ msgid \"\"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_unit(false) {
                Ok(None) => (),
                r => panic!("Unexpected result for the test on \"non-message\": {:?}", r),
            }
        }

        {
            let source = "msgid \"\"\nmsgstr \"Something\"";
            let mut reader = make_reader(source.as_bytes(), &parser);

            match reader.read_unit(false) {
                Err(err) => assert_eq!(
                    format!("{:?}", err),
                    "Unexpected error at line 1: Source should not be empty"
                ),
                Ok(r) => panic!("Unexpected result for the test on empty messages: {:?}", r),
            }
        }
    }

    #[test]
    fn test_func_next_unit() {
        let source = "msgid \"my-id\"\nmsgstr \"my-text\"\nmsgid \"with-error\"\nmsgstr \"";
        let parser = PoParser::new();
        let mut reader = make_reader(source.as_bytes(), &parser);

        match reader.next_unit(false) {
            Some(Ok(unit)) => {
                assert_eq!(unit.message.get_id(), "my-id");
                assert_eq!(unit.message.get_text(), "my-text");
            }
            v => panic!("Unexpected result for any message: {:?}", v),
        }

        match reader.next_unit(false) {
            Some(Err(err)) => assert_eq!(format!("{:?}", err), "Parse error at line 5, got ‘msgstr \"’"),
            v => panic!("Unexpected result for test on error: {:?}", v),
        }

        match reader.next_unit(false) {
            None => (),
            v => panic!("Unexpected result for test on the end of source: {:?}", v),
        }
    }

    #[test]
    fn test_func_new_with_error() {
        let source = "msgid \"\"\nmsgstr \"\"\n\"Plural-Forms: plural=1+\"";
        let parser = PoParser::new();
        let reader = PoReader::new(source.as_bytes(), &parser);

        match reader {
            Err(err) => assert_eq!(
                format!("{:?}", err),
                "Unexpected error: Bad value list definition: `plural=1+`"
            ),
            Ok(v) => panic!(
                "Unexpected result: forms={:?}, notes={:?}, headers={:?}, next={:?}",
                v.plural_forms, v.header_notes, v.header_properties, v.next_unit,
            ),
        }
    }

    #[test]
    fn test_func_new_normal() {
        let source = make_source();
        let parser = PoParser::new();

        match PoReader::new(source.as_bytes(), &parser) {
            Ok(reader) => {
                assert_eq!(reader.target_language().as_ref(), "fr");
                assert_eq!(
                    reader.header_notes(),
                    &vec![Note::new(Origin::Translator, String::from("Any note"))]
                );

                assert_eq!(
                    reader.header_comments(),
                    &vec![
                        Comment::new('$', String::from("Any comment 1")),
                        Comment::new('&', String::from("Any comment 2")),
                    ]
                );

                assert_eq!(
                    reader.header_properties(),
                    &[("Any-Header", "Value"), ("Language", "fr"),]
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<HashMap<_, _>>()
                );

                match reader.plural_forms {
                    None => (),
                    Some(forms) => panic!("Unexpected forms: {:?}", forms),
                }
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_trait_iterator_normal() {
        let source = make_source();
        let parser = PoParser::new();

        match PoReader::new(source.as_bytes(), &parser) {
            Ok(mut reader) => {
                match reader.next() {
                    Some(Ok(unit)) => {
                        assert_eq!(unit.message.get_id(), "Hello, world !");
                        assert_eq!(unit.message.get_text(), "Salut, tout le monde !");
                    }
                    r => panic!("Unexpected result after the first call of `next()`: {:?}", r),
                }

                match reader.next() {
                    None => (),
                    Some(r) => panic!("Unexpected result after the second call of `next()`: {:?}", r),
                }
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_trait_iterator_with_error() {
        let source = "msgid \"msg\"\nmsgstr \"text\"\n\n#? xxx\nmsgid \"";
        let parser = PoParser::new();

        match PoReader::new(source.as_bytes(), &parser) {
            Ok(mut reader) => {
                match reader.next() {
                    Some(Ok(unit)) => {
                        assert_eq!(unit.message.get_id(), "msg");
                        assert_eq!(unit.message.get_text(), "text");
                    }
                    r => panic!("Unexpected result after the first call of `next()`: {:?}", r),
                }

                match reader.next() {
                    Some(Err(err)) => assert_eq!(format!("{:?}", err), "Parse error at line 6, got ‘msgid \"’"),
                    r => panic!("Unexpected result after the second call of `next()`: {:?}", r),
                }

                match reader.next() {
                    None => (),
                    Some(r) => panic!("Unexpected result after the third call of `next()`: {:?}", r),
                }
            }
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }
}
