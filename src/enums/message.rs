use crate::plural::Plural;

/// String wrapper possibly with plural variants.
///
/// This is used for source and target strings in translation Unit.
#[derive(Clone, Debug)]
pub enum Message {
    /// Simple message independent of any count.
    Simple { id: String, text: Option<String> },

    /// Count-dependent message with some variants. Must have at least variant for Other.
    Plural(Plural),
}

impl Message {
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Message::Simple { id, .. } => id.is_empty(),
            _ => false,
        }
    }

    pub fn is_simple(&self) -> bool {
        match self {
            Message::Simple { id, .. } => !id.is_empty(),
            _ => false,
        }
    }

    pub fn is_plural(&self) -> bool {
        match self {
            Message::Plural(_) => true,
            _ => false,
        }
    }

    pub fn is_blank(&self) -> bool {
        match self {
            Message::Simple { text, .. } => text.as_ref().map_or(true, |s| s.is_empty()),
            Message::Plural(m) => m.is_blank(),
        }
    }

    pub fn get_id(&self) -> &str {
        match self {
            Message::Simple { id, .. } => id.as_str(),
            Message::Plural(p) => p.singular(),
        }
    }

    pub fn get_text(&self) -> &str {
        match self {
            Message::Simple { text, .. } => text.as_ref().map(|s| s.as_str()).unwrap_or_default(),
            Message::Plural(p) => p.first(),
        }
    }

    pub fn get_plural_id(&self) -> Option<&str> {
        match self {
            Message::Plural(p) => Some(p.plural()),
            _ => None,
        }
    }

    pub fn get_plural_text(&self, count: usize) -> Option<&str> {
        match self {
            Message::Plural(p) => p.get(count),
            Message::Simple { text, .. } => text.as_ref().map(|s| s.as_str()),
        }
    }

    pub fn plural(&self) -> Option<&Plural> {
        match self {
            Message::Plural(p) => Some(p),
            _ => None,
        }
    }
}

impl Default for Message {
    fn default() -> Message {
        Message::Simple {
            id: String::new(),
            text: None,
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Message::Simple { id: li, text: lt }, Message::Simple { id: ri, text: rt }) => {
                (li == ri)
                    && match (lt, rt) {
                        (None, None) => true,
                        (None, Some(s)) if s.is_empty() => true,
                        (Some(s), None) if s.is_empty() => true,
                        (Some(l), Some(r)) => r == l,
                        _ => false,
                    }
            }
            (Message::Plural(l), Message::Plural(r)) => (l.singular() == r.singular()) && (l.plural() == r.plural()),
            _ => false,
        }
    }
}

impl Eq for Message {}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use crate::plural::PluralForms;
    use std::rc::Rc;

    #[test]
    fn test_enum() {
        let msg_simple = Message::Simple {
            id: String::from("id"),
            text: None,
        };

        assert_eq!(msg_simple.clone(), msg_simple);
        assert_eq!(format!("{:?}", msg_simple), "Simple { id: \"id\", text: None }");

        let plural = Plural::new(String::from("id"), String::from("val"), vec![], None);
        let msg_plural = Message::Plural(plural.clone());

        assert_eq!(msg_plural.clone(), msg_plural);
        assert_eq!(format!("{:?}", msg_plural), format!("Plural({:?})", plural));

        assert_ne!(msg_simple, msg_plural);

        let msg = Message::Simple {
            id: String::from("id"),
            text: Some(String::new()),
        };

        assert_eq!(msg_simple, msg);
        assert_eq!(msg, msg_simple);
        assert_ne!(
            Message::Simple {
                id: String::from("id"),
                text: Some(String::from("txt"))
            },
            msg_simple
        );
    }

    #[test]
    fn test_trait_default() {
        match Message::default() {
            Message::Simple { id, text } => {
                assert_eq!(id, "");
                assert_eq!(text, None);
            }
            msg => panic!("Bad message: {:?}", msg),
        }
    }

    #[test]
    fn test_func_is_empty() {
        assert!(Message::default().is_empty(), "Default message should be empty");

        let msg = Message::Simple {
            id: String::new(),
            text: None,
        };

        assert!(msg.is_empty(), "Simple with no id should be empty");

        let msg = Message::Simple {
            id: String::new(),
            text: Some(String::from("Something")),
        };

        assert!(
            msg.is_empty(),
            "Simple with no id should be empty even if text is not empty"
        );

        let msg = Message::Simple {
            id: String::from("Something"),
            text: None,
        };

        assert!(
            !msg.is_empty(),
            "Simple with id should not be empty even if text is empty"
        );

        let msg = Message::Plural(Plural::new_empty());

        assert!(!msg.is_empty(), "Plural (even if empty) should not be empty");
    }

    #[test]
    fn test_func_is_simple() {
        assert!(Message::default().is_empty(), "Default message should be empty");

        let msg = Message::Simple {
            id: String::new(),
            text: Some(String::from("Something")),
        };

        assert!(
            !msg.is_simple(),
            "Simple with no id should not be simple even if text is not empty"
        );

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::from("Something")),
        };

        assert!(msg.is_simple(), "Simple with id should be simple");

        let msg = Message::Simple {
            id: String::from("Something"),
            text: None,
        };

        assert!(msg.is_simple(), "Simple with id should be simple even if text is empty");

        let msg = Message::Plural(Plural::new_empty());

        assert!(!msg.is_simple(), "Plural should not be simple");
    }

    #[test]
    fn test_func_is_plural() {
        assert!(!Message::default().is_plural(), "Default message should not be plural");
        assert!(
            Message::Plural(Plural::new_empty()).is_plural(),
            "This should be plural"
        )
    }

    #[test]
    fn test_func_is_blank() {
        assert!(Message::default().is_blank(), "Default message should be blank");

        let msg = Message::Simple {
            id: String::from("Something"),
            text: None,
        };

        assert!(msg.is_blank(), "Simple with no text should be blank");

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::new()),
        };

        assert!(msg.is_blank(), "Simple with empty text should be blank");

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::from("Here")),
        };

        assert!(!msg.is_blank(), "Simple with text should not be blank");

        let msg = Message::Plural(Plural::new_empty());

        assert!(msg.is_blank(), "Empty plural should be blank");

        let msg = Message::Plural(Plural::with_no_forms(String::from("Something"), None));

        assert!(msg.is_blank(), "Non-empty plural should not be blank");
    }

    #[test]
    fn test_func_get_id() {
        assert_eq!(Message::default().get_id(), "");

        let msg = Message::Simple {
            id: String::from("Something"),
            text: None,
        };

        assert_eq!(msg.get_id(), "Something");

        let msg = Message::Plural(Plural::new_empty());

        assert_eq!(msg.get_id(), "");

        let msg = Message::Plural(Plural::with_no_forms(String::from("Something"), None));

        assert_eq!(msg.get_id(), "Something");
    }

    #[test]
    fn test_func_get_text() {
        assert_eq!(Message::default().get_text(), "");

        let msg = Message::Simple {
            id: String::new(),
            text: Some(String::from("Something")),
        };

        assert_eq!(msg.get_text(), "Something");

        let msg = Message::Plural(Plural::new_empty());

        assert_eq!(msg.get_text(), "");

        let values = vec![String::from("Something")];
        let msg = Message::Plural(Plural::new(String::new(), String::new(), values, None));

        assert_eq!(msg.get_text(), "Something");
    }

    #[test]
    fn test_func_get_plural_id() {
        assert_eq!(Message::default().get_plural_id(), None);

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::from("Here")),
        };

        assert_eq!(msg.get_plural_id(), None);

        let msg = Message::Plural(Plural::new(String::new(), String::from("Something"), vec![], None));

        assert_eq!(msg.get_plural_id(), Some("Something"));
    }

    #[test]
    fn test_func_get_plural_text() {
        assert_eq!(Message::default().get_plural_text(100), None);

        let msg = Message::Simple {
            id: String::from("Something"),
            text: None,
        };

        assert_eq!(msg.get_plural_text(100), None);

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::from("Here")),
        };

        assert_eq!(msg.get_plural_text(100), Some("Here"));

        let values = vec![String::from("Something-1"), String::from("Something-2")];
        let forms = Rc::new(PluralForms::for_tests_shift());
        let msg = Message::Plural(Plural::new(String::new(), String::new(), values, Some(forms)));

        assert_eq!(msg.get_plural_text(50), None);
        assert_eq!(msg.get_plural_text(100), Some("Something-1"));
        assert_eq!(msg.get_plural_text(101), Some("Something-2"));
    }

    #[test]
    fn test_func_plural() {
        assert!(
            Message::default().plural().is_none(),
            "Default message should not have plural"
        );

        let msg = Message::Simple {
            id: String::from("Something"),
            text: Some(String::from("Here")),
        };

        assert!(
            msg.plural().is_none(),
            "The method `Message::plural` for messages without plural should return None"
        );

        let plural = Plural::new_empty();
        let msg = Message::Plural(plural.clone());

        assert_eq!(msg.plural(), Some(&plural));

        let plural = Plural::new(String::from("Something"), String::from("Here"), vec![], None);
        let msg = Message::Plural(plural.clone());

        assert_eq!(msg.plural(), Some(&plural));
    }
}
// no-coverage:stop
