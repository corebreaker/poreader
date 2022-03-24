// no-coverage:start
use locale_config::LanguageRange;
use poreader::{error::Error, note::Note, CatalogueReader, Message, Origin, PoParser, State};

static SAMPLE_PO: &'static str = r###"
msgid ""
msgstr ""
"Project-Id-Version: poreader test\n"
"PO-Revision-Date: 2017-04-24 21:39+02:00\n"
"Last-Translator: Frédéric Meyer <frederic.meyer.77@gmail.com>\n"
"Language-Team: French\n"
"Language: fr\n"
"MIME-Version: 1.0\n"
"Header1: Value1\n"
"Header2: ValueX\n"
"Header1: Value2\n"
"Content-Type: text/plain; charset=ISO-8859-2\n"
"Content-Transfer-Encoding: 8bit\n"
"Plural-Forms: nplurals=3;\n"
"Plural-Forms: plural=(n==1) ? 0 : (n>=2 && n<=4) ? 1 : 2;\n"

msgid "Simple message"
msgstr "Un simple message"

#. Extracted comment
#  Translator comment
#: Location:42  Another:69
#, fuzzy
#| msgctxt "ConTeXt"
#| msgid "Previous message"
msgctxt "ConTeXt"
msgid "Changed message"
msgstr "Message\n"
"changé"

msgid "Untranslated message"
msgstr ""

msgid "A message with several translations"
msgid_plural "Some messages with several translations"
msgstr[0] "Un message avec plusieurs traductions"
msgstr[1] "Quelques messages avec plusieurs traductions"
msgstr[2] "Des messages avec plusieurs traductions"

# Another comment
#~ msgid "Obsolete message"
#~ msgstr "Message obsolète"

"###;

macro_rules! a_str {
    ($v:literal) => {
        String::from($v)
    };
}

#[test]
fn integration_test() -> Result<(), Error> {
    let input = SAMPLE_PO.as_bytes();
    let parser = PoParser::new();
    let mut reader = parser.parse(input)?;
    let lang = LanguageRange::new("fr").unwrap();

    assert_eq!(reader.target_language(), &lang);
    assert_eq!(
        reader.header_properties().get("Project-Id-Version"),
        Some(&a_str!("poreader test"))
    );

    {
        assert_eq!(reader.header_properties().get("Header0"), None);
        assert_eq!(
            reader.header_properties().get("Header1"),
            Some(&a_str!("Value1 Value2"))
        );
        assert_eq!(reader.header_properties().get("Header2"), Some(&a_str!("ValueX")));
    }

    {
        let u = reader.next().unwrap().unwrap();

        assert_eq!(u.prev_context(), None);
        assert_eq!(u.context(), None);

        {
            let msg = u.prev_message();

            assert!(msg.get_id().is_empty(), "It should be empty");
        }

        {
            let msg = u.message();

            assert!(msg.is_simple(), "It should be singular");
            assert_eq!(msg.get_id(), "Simple message");
            assert_eq!(msg.get_text(), "Un simple message");
            assert_eq!(msg.get_plural_id(), None);
            assert_eq!(msg.get_plural_text(1), Some("Un simple message"));
        }

        assert!(u.notes().is_empty(), "There should have no note");
        assert!(u.locations().is_empty(), "There should have no location");
        assert!(u.is_translated(), "It should be translated");
        assert!(!u.is_obsolete(), "It should not be obsolete");
        assert_eq!(u.state(), State::Final);
    }

    {
        let u = reader.next().unwrap().unwrap();

        assert_eq!(u.prev_context(), Some("ConTeXt"));
        assert_eq!(u.context(), Some("ConTeXt"));

        {
            let msg = u.prev_message();

            assert!(msg.is_simple(), "It should be singular");
            assert_eq!(msg.get_id(), "Previous message");
        }

        {
            let msg = u.message();

            assert!(msg.is_simple(), "It should be singular");
            assert_eq!(msg.get_id(), "Changed message");
            assert_eq!(msg.get_text(), "Message\nchangé");
        }

        assert_eq!(
            u.notes(),
            &vec![
                Note::new(Origin::Developer, a_str!("Extracted comment")),
                Note::new(Origin::Translator, a_str!("Translator comment")),
            ]
        );

        assert_eq!(u.locations(), &vec![a_str!("Location:42"), a_str!("Another:69"),]);

        assert!(!u.is_translated(), "It should not be translated");
        assert!(!u.is_obsolete(), "It should not be obsolete");
        assert_eq!(u.state(), State::NeedsWork);
    }

    {
        let u = reader.next().unwrap().unwrap();
        let empty_msg = Message::default();
        let msg = Message::Simple {
            id: a_str!("Untranslated message"),
            text: None,
        };

        assert_eq!(u.context(), None);
        assert_eq!(u.message(), &msg);
        assert_eq!(u.prev_context(), None);
        assert_eq!(u.prev_message(), &empty_msg);
        assert!(u.notes().is_empty(), "There should be no note");
        assert!(u.locations().is_empty(), "There should be no location");
        assert_eq!(u.state(), State::Empty);
        assert!(!u.is_translated(), "This entry should not be translated");
        assert!(!u.is_obsolete(), "This entry should not be obsolete");
    }

    {
        let u = reader.next().unwrap().unwrap();
        let msg = u.message();

        assert_eq!(msg.get_id(), "A message with several translations");
        assert_eq!(msg.get_plural_id(), Some("Some messages with several translations"));
        assert_eq!(msg.get_plural_text(1), Some("Un message avec plusieurs traductions"));
        assert_eq!(msg.get_plural_text(10), Some("Des messages avec plusieurs traductions"));
        assert_eq!(
            msg.get_plural_text(3),
            Some("Quelques messages avec plusieurs traductions")
        );
    }

    {
        let u = reader.next().unwrap().unwrap();
        let empty_msg = Message::default();
        let msg = Message::Simple {
            id: a_str!("Obsolete message"),
            text: Some(a_str!("Message obsolète")),
        };

        assert!(u.is_obsolete(), "This entry should be obsolete");
        assert!(u.is_translated(), "This entry should be translated");
        assert!(u.locations().is_empty(), "There should be no location");
        assert_eq!(u.context(), None);
        assert_eq!(u.message(), &msg);
        assert_eq!(u.prev_context(), None);
        assert_eq!(u.prev_message(), &empty_msg);
        assert_eq!(u.state(), State::Final);
        assert_eq!(
            u.notes().as_slice(),
            &[Note::new(Origin::Translator, a_str!("Another comment"))]
        );
    }

    assert!(reader.next().is_none(), "There should be no other entry in the stream");

    Ok(())
}

// no-coverage:stop
