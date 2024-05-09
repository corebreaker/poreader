use super::Decoder;
use crate::{
    error::Error,
    plural::{Plural, PluralForms},
    unit::Unit,
    Message,
};
use std::rc::Rc;

pub(crate) struct MessageExtractor<'r, D: Decoder> {
    unit: Unit,
    decoder: &'r mut D,
    plural_forms: Option<Rc<PluralForms>>,
}

impl<'r, D: Decoder> MessageExtractor<'r, D> {
    pub(super) fn new(unit: Unit, decoder: &'r mut D, plural_forms: Option<Rc<PluralForms>>) -> Self {
        MessageExtractor {
            unit,
            decoder,
            plural_forms,
        }
    }

    pub(super) fn parse_message_fields(mut self, first: bool) -> Result<Option<Unit>, Error> {
        // previous context
        let prev_context = self.parse_msg("|msgctxt")?;

        // previous source
        let prev_msgid = self.parse_msg("|msgid")?;
        let prev_msgid_pl = match prev_msgid {
            Some(_) => self.parse_msg("|msgid_plural")?,
            None => None,
        };

        // previous message (simple message with empty text)
        let prev_message = self.new_previous(prev_msgid, prev_msgid_pl);

        // context
        let context = self.parse_msg("msgctxt")?;

        // source
        let msgid = self.parse_msg("msgid")?;

        if (!first) && msgid.is_none() {
            self.expected("msgid")?;

            return Ok(None);
        }

        // plural
        let msgid_pl = self.parse_msg("msgid_plural")?;

        let message = match self.new_message(msgid, msgid_pl)? {
            Some(msg) => msg,
            None => {
                return Ok(None);
            }
        };

        // apply result
        self.unit.prev_context = prev_context;
        self.unit.prev_message = prev_message;
        self.unit.context = context;
        self.unit.message = message;

        Ok(Some(self.unit))
    }

    fn new_message(&mut self, msgid: Option<String>, msgid_pl: Option<String>) -> Result<Option<Message>, Error> {
        let singular = match msgid {
            None => {
                return Ok(Some(Message::default()));
            }
            Some(singular) => match msgid_pl {
                None => singular,
                Some(plural) => {
                    let mut values = vec![];
                    let forms = self.plural_forms();
                    let count = forms.as_ref().map_or(2, |f| f.get_count());

                    for i in 0..count {
                        if let Some(v) = self.parse_msg(&format!("msgstr[{}]", i))? {
                            values.push(v);
                        };
                    }

                    return Ok(if values.is_empty() {
                        self.expected("msgstr[0]")?;

                        None
                    } else {
                        Some(Message::Plural(Plural::new(singular, plural, values, forms)))
                    });
                }
            },
        };

        Ok(match self.parse_msg("msgstr")? {
            Some(text) => Some(Message::Simple {
                id: singular,
                text: if text.is_empty() { None } else { Some(text) },
            }),
            None => {
                self.expected("msgstr")?;

                None
            }
        })
    }

    fn new_previous(&self, msgid: Option<String>, msgid_pl: Option<String>) -> Message {
        match msgid {
            None => Message::default(),
            Some(singular) => match msgid_pl {
                None => Message::Simple {
                    id: singular,
                    text: None,
                },
                Some(plural) => Message::Plural(Plural::new(singular, plural, vec![], self.plural_forms())),
            },
        }
    }

    fn parse_msg(&mut self, tag: &str) -> Result<Option<String>, Error> {
        self.decoder.parse_msg(tag, &self.unit)
    }

    fn expected(&mut self, exp: &str) -> Result<(), Error> {
        self.decoder.expected(exp)
    }

    fn plural_forms(&self) -> Option<Rc<PluralForms>> {
        self.plural_forms.as_ref().map(Rc::clone)
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::{
        super::decoder::tests::{TestAction::*, TestDecoder},
        *,
    };

    use crate::error::Error;

    impl<'r, D: Decoder> MessageExtractor<'r, D> {
        pub(crate) fn for_tests_zero(decoder: &'r mut D) -> Self {
            Self::new(Unit::for_tests_empty(), decoder, None)
        }

        pub(crate) fn for_tests_no_forms(decoder: &'r mut D) -> Self {
            Self::new(Unit::for_tests_normal(), decoder, None)
        }

        pub(crate) fn for_tests_empty(decoder: &'r mut D) -> Self {
            Self::new(
                Unit::for_tests_empty(),
                decoder,
                Some(Rc::new(PluralForms::for_tests_empty())),
            )
        }

        pub(crate) fn for_tests_shift(decoder: &'r mut D) -> Self {
            Self::new(
                Unit::for_tests_normal(),
                decoder,
                Some(Rc::new(PluralForms::for_tests_shift())),
            )
        }

        pub(crate) fn for_tests_normal(decoder: &'r mut D) -> Self {
            Self::new(
                Unit::for_tests_normal(),
                decoder,
                Some(Rc::new(PluralForms::for_tests_simple())),
            )
        }
    }

    #[test]
    fn test_func_parse_message_fields_with_errors_on_new_message() {
        let err = Error::Unexpected(123, String::from("Error"));
        let err_msg = format!("{:?}", err);
        let mut decoder = TestDecoder::with_values([("msgid", ActOk("Something")), ("msgid_plural", ActErr(err))]);

        let msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.parse_message_fields(false) {
            Err(err) => assert_eq!(format!("{:?}", err), err_msg),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_message_fields_with_errors_on_expected() {
        let err = Error::Unexpected(123, String::from("Error"));
        let err_msg = format!("{:?}", err);
        let mut decoder = TestDecoder::with_values([("msgid", ActDelayed(err))]);

        decoder.set_message(String::from("msgid"));

        let msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.parse_message_fields(false) {
            Err(err) => assert_eq!(format!("{:?}", err), err_msg),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_message_fields_with_errors_on_parse_msg() {
        let err_msg = format!("{:?}", Error::Unexpected(123, String::from("Error")));
        let mut decoder = TestDecoder::new();

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("|msgctxt", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("|msgid", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("|msgid", ActOk("Something")), ("|msgid_plural", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("msgctxt", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("msgid", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        {
            let err = Error::Unexpected(123, String::from("Error"));

            decoder.push_values([("msgid", ActOk("Something")), ("msgid_plural", ActErr(err))]);

            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(false) {
                Err(err) => assert_eq!(format!("{:?}", err), err_msg),
                r => panic!("Unexpected result: {:?}", r),
            }
        }
    }

    #[test]
    fn test_func_parse_message_fields_singular() {
        let mut decoder = TestDecoder::with_values([
            ("|msgctxt", ActOk("prev-ctx")),
            ("|msgid", ActOk("prev-id")),
            ("msgctxt", ActOk("my-ctx")),
            ("msgid", ActOk("my-id")),
            ("msgstr", ActOk("my-text")),
        ]);

        let msg = MessageExtractor::for_tests_no_forms(&mut decoder);

        match msg.parse_message_fields(true) {
            Ok(Some(unit)) => {
                assert_eq!(unit.prev_context(), Some("prev-ctx"));
                assert_eq!(unit.context(), Some("my-ctx"));

                let msg = unit.prev_message();

                assert!(msg.is_simple(), "Previous message should be simple");
                assert_eq!(msg.get_id(), "prev-id");
                assert_eq!(msg.get_text(), "");
                assert_eq!(msg.get_plural_id(), None);
                assert_eq!(msg.get_plural_text(1), None);
                assert_eq!(msg.get_plural_text(24), None);

                let msg = unit.message();

                assert!(msg.is_simple(), "Message should be simple");
                assert_eq!(msg.get_id(), "my-id");
                assert_eq!(msg.get_text(), "my-text");
                assert_eq!(msg.get_plural_id(), None);
                assert_eq!(msg.get_plural_text(1), Some("my-text"));
                assert_eq!(msg.get_plural_text(24), Some("my-text"));
            }
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_message_fields_plural() {
        let mut decoder = TestDecoder::with_values([
            ("|msgctxt", ActOk("prev-ctx")),
            ("|msgid", ActOk("prev-id")),
            ("|msgid_plural", ActOk("prev-plural")),
            ("msgctxt", ActOk("my-ctx")),
            ("msgid", ActOk("my-id")),
            ("msgid_plural", ActOk("my-plural")),
            ("msgstr[0]", ActOk("my-text-1")),
            ("msgstr[1]", ActOk("my-text-2")),
            ("msgstr[2]", ActOk("my-text-3")),
        ]);

        let msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.parse_message_fields(true) {
            Ok(Some(unit)) => {
                assert_eq!(unit.prev_context(), Some("prev-ctx"));
                assert_eq!(unit.context(), Some("my-ctx"));

                let msg = unit.prev_message();

                assert!(msg.is_plural(), "Previous message should be plural");
                assert_eq!(msg.get_id(), "prev-id");
                assert_eq!(msg.get_text(), "");
                assert_eq!(msg.get_plural_id(), Some("prev-plural"));
                assert_eq!(msg.get_plural_text(1), None);
                assert_eq!(msg.get_plural_text(24), None);

                let msg = unit.message();

                assert!(msg.is_plural(), "Message should be plural");
                assert_eq!(msg.get_id(), "my-id");
                assert_eq!(msg.get_text(), "my-text-1");
                assert_eq!(msg.get_plural_id(), Some("my-plural"));
                assert_eq!(msg.get_plural_text(1), Some("my-text-1"));
                assert_eq!(msg.get_plural_text(24), Some("my-text-2"));
                assert_eq!(msg.get_plural_text(114), Some("my-text-3"));
            }
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_message_fields_for_empty_messages() {
        let mut decoder = TestDecoder::with_values([("msgstr", ActOk("some text"))]);

        {
            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(true) {
                Ok(Some(unit)) => {
                    assert!(unit.message.is_empty(), "With no `msgid`, the unit should be empty");
                    assert_eq!(unit.message.get_text(), "");
                }
                r => panic!("Unexpected result: {:?}", r),
            }
        }

        decoder.push_values([("msgid", ActOk(""))]);

        {
            let msg = MessageExtractor::for_tests_normal(&mut decoder);

            match msg.parse_message_fields(true) {
                Ok(Some(unit)) => {
                    assert!(unit.message.is_empty(), "With empty `msgid`, the unit should be empty");
                    assert_eq!(unit.message.get_text(), "some text");
                }
                r => panic!("Unexpected result: {:?}", r),
            }
        }
    }

    #[test]
    fn test_func_parse_message_fields_returns_none() {
        let mut decoder = TestDecoder::with_values([]);
        let msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.parse_message_fields(false) {
            Ok(None) => (),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_parse_msg() {
        let mut decoder = TestDecoder::new();
        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.parse_msg("tag1") {
            Ok(Some(r)) => {
                assert_eq!(
                    msg.unit.locations,
                    vec![
                        String::from("EmptyFile1:11"),
                        String::from("EmptyFile2:22"),
                        String::from("EmptyFile3:33"),
                    ]
                );

                assert_eq!(msg.decoder.log(), &[format!("Message: {}/tag1", r)]);
            }
            r => panic!("Bad result for tag1: {:?}", r),
        }

        msg.decoder.inc();
        match msg.parse_msg("tag2") {
            Ok(Some(r)) => assert_eq!(
                msg.decoder.log(),
                &[String::from("Message: message-0/tag1"), format!("Message: {}/tag2", r),]
            ),
            r => panic!("Bad result for tag2: {:?}", r),
        }

        msg.decoder.inc();
        msg.decoder.push_values([]);
        match msg.parse_msg("tag3") {
            Ok(None) => {
                assert_eq!(
                    msg.decoder.log(),
                    &[
                        String::from("Message: message-0/tag1"),
                        String::from("Message: message-1/tag2"),
                        String::from("Message: message-2/tag3"),
                    ]
                );
            }
            r => panic!("Bad result for tag3: {:?}", r),
        }

        let estr = "An error";

        msg.decoder.set_error(Error::Unexpected(123, estr.to_string()));
        match msg.parse_msg("tag4") {
            Err(err) => {
                assert_eq!(format!("{}", err), format!("Unexpected error at line 123: {}", estr));
            }
            r => panic!("Bad result for tag4: {:?}", r),
        }
    }

    #[test]
    fn test_func_expected() {
        let mut decoder = TestDecoder::new();
        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.expected("---") {
            Ok(()) => assert!(msg.decoder.log().is_empty(), "Decoder log should be empty"),
            r => panic!("Bad result for first call of `expected`: {:?}", r),
        }

        let mut log = vec![String::from("Expected: message-1")];

        msg.decoder.inc();
        match msg.expected("message-1") {
            Ok(()) => assert_eq!(msg.decoder.log(), &log),
            r => panic!("Bad result for second call of `expected`: {:?}", r),
        }

        let estr = "An error";

        log.push(String::from("Expected: message-2"));
        msg.decoder.inc();
        msg.decoder.set_error(Error::Unexpected(123, estr.to_string()));
        match msg.expected("message-2") {
            Err(err) => {
                assert_eq!(format!("{}", err), format!("Unexpected error at line 123: {}", estr));
                assert_eq!(msg.decoder.log(), &log);
            }
            r => panic!("Bad result for third call of `expected`: {:?}", r),
        }
    }

    #[test]
    fn test_func_plural_forms() {
        let mut decoder = TestDecoder::new();
        let msg = MessageExtractor::for_tests_zero(&mut decoder);

        assert!(
            msg.plural_forms().is_none(),
            "In case of no forms, the method `plural_forms` should return none"
        );

        let msg = MessageExtractor::for_tests_empty(&mut decoder);
        if let Some(forms) = msg.plural_forms() {
            assert_eq!(forms.get_value(0), None);
            assert_eq!(forms.get_value(100), None);
            assert_eq!(forms.get_count(), 0);
            assert_eq!(forms.get_formula(), "");
            assert_eq!(forms.get_definition(), "");
        } else {
            panic!("None should be returned");
        }

        let msg = MessageExtractor::for_tests_shift(&mut decoder);
        if let Some(forms) = msg.plural_forms() {
            assert_eq!(forms.get_value(50), None);
            assert_eq!(forms.get_value(500), None);
            assert_eq!(forms.get_value(150), Some(50));
            assert_eq!(forms.get_count(), 200);
            assert_eq!(forms.get_formula(), "n-100");
        } else {
            panic!("None should be returned");
        }

        let msg = MessageExtractor::for_tests_normal(&mut decoder);
        if let Some(forms) = msg.plural_forms() {
            assert_eq!(forms.get_value(1), Some(0));
            assert_eq!(forms.get_value(24), Some(1));
            assert_eq!(forms.get_value(10), Some(2));
            assert_eq!(forms.get_count(), 3);
        } else {
            panic!("None should be returned");
        }
    }

    #[test]
    fn test_func_new_previous() {
        let mut decoder = TestDecoder::new();
        let msg = MessageExtractor::for_tests_no_forms(&mut decoder);

        assert_eq!(msg.new_previous(None, None), Message::default());
        assert_eq!(
            msg.new_previous(None, Some(String::from("Something"))),
            Message::default()
        );

        let prev = msg.new_previous(Some(String::from("my-msg")), None);
        assert!(
            !prev.is_empty(),
            "New previous message without plural should not be empty"
        );
        assert!(prev.is_blank(), "New previous message without plural should be blank");
        assert!(prev.is_simple(), "New previous message without plural should be simple");
        assert!(
            !prev.is_plural(),
            "New previous message without plural should not be plural"
        );
        assert_eq!(prev.get_id(), "my-msg");
        assert_eq!(prev.get_text(), "");
        assert_eq!(prev.get_plural_id(), None);
        assert_eq!(prev.get_plural_text(0), None);
        assert_eq!(prev.get_plural_text(2), None);

        let prev = msg.new_previous(Some(String::from("my-msg")), Some(String::from("my-plural")));
        assert!(!prev.is_empty(), "New previous message with plural should not be empty");
        assert!(prev.is_blank(), "New previous message with plural should be blank");
        assert!(
            !prev.is_simple(),
            "New previous message with plural should not be simple"
        );
        assert!(prev.is_plural(), "New previous message with plural should be plural");
        assert_eq!(prev.get_id(), "my-msg");
        assert_eq!(prev.get_text(), "");
        assert_eq!(prev.get_plural_id(), Some("my-plural"));
        assert_eq!(prev.get_plural_text(0), None);
        assert_eq!(prev.get_plural_text(2), None);
    }

    #[test]
    fn test_func_new_message_for_error_from_none() {
        let mut decoder = TestDecoder::with_values([]);

        decoder.set_message(String::from("msgstr"));
        decoder.set_command(String::from("@DoError"));

        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.new_message(Some(String::from("MyMsg0")), None) {
            Err(err) => assert_eq!(
                format!("{:?}", err),
                "Unexpected error at line 210: From command `@DoError`"
            ),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_error_from_singular() {
        let mut decoder = TestDecoder::new();
        let err = Error::Unexpected(123, String::from("But this is expected"));
        let err_msg = format!("{:?}", err);

        decoder.set_error(err);

        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.new_message(Some(String::from("MyMsg1")), None) {
            Err(err) => assert_eq!(format!("{:?}", err), err_msg),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_error_from_plural_on_parse() {
        let mut decoder = TestDecoder::new();
        let err = Error::Unexpected(123, String::from("But this is expected"));
        let err_msg = format!("{:?}", err);

        decoder.set_error(err);

        let mut msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.new_message(Some(String::from("MyMsg2")), Some(String::from("MyText1"))) {
            Err(err) => assert_eq!(format!("{:?}", err), err_msg),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_error_from_plural_on_expected() {
        let mut decoder = TestDecoder::new();

        decoder.set_message(String::from("msgstr[0]"));
        decoder.set_command(String::from("@DoError"));

        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.new_message(Some(String::from("MyMsg3")), Some(String::from("MyText2"))) {
            Err(err) => assert_eq!(
                format!("{:?}", err),
                "Unexpected error at line 210: From command `@DoError`"
            ),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_singular() {
        let mut decoder = TestDecoder::new();
        let mut msg = MessageExtractor::for_tests_zero(&mut decoder);

        match msg.new_message(Some(String::from("my-msg")), None) {
            Ok(Some(Message::Simple { id, text })) => {
                assert_eq!(id, "my-msg");
                assert_eq!(text, Some(String::from("message-0")));
            }
            r => panic!("Bad result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_no_plural_forms() {
        let mut decoder = TestDecoder::with_values([]);
        let mut msg = MessageExtractor::for_tests_no_forms(&mut decoder);

        match msg.new_message(Some(String::from("my-id")), Some(String::from("my-plural"))) {
            Ok(None) => (),
            r => panic!("Unexpected result: {:?}", r),
        }
    }

    #[test]
    fn test_func_new_message_for_plural() {
        let mut decoder = TestDecoder::new();

        decoder.set_command(String::from("@DoInc"));

        let mut msg = MessageExtractor::for_tests_normal(&mut decoder);

        match msg.new_message(Some(String::from("my-id")), Some(String::from("my-plural"))) {
            Ok(Some(Message::Plural(plural))) => {
                let values = vec![
                    String::from("message-1"),
                    String::from("message-2"),
                    String::from("message-3"),
                ];

                assert_eq!(plural.singular(), "my-id");
                assert_eq!(plural.plural(), "my-plural");
                assert_eq!(plural.values(), &values);
            }
            r => panic!("Unexpected result: {:?}", r),
        }
    }
}
// no-coverage:stop
