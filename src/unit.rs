use super::{comment::Comment, note::Note, Message, State};
use std::collections::HashSet;

/// Elementary unit of translation.
///
/// A translation unit contains:
///
/// - One *source* string, the original message.
/// - At most one *target* string, the translated message.
/// - Optional *context* string that disambiguates the original.
/// - A status. This indicates whether the unit is usable in the software.
///
/// Additionally, it can also contain:
///  - Notes, from developer or translator.
///  - References back into the source where the unit is used.
///  - Previous source and context if the target is automatic suggestion from fuzzy matching.
///  - Obsolete flag, indicating the unit is not currently in use.
///  - Comments (which are not notes, locations and flags).
#[derive(Clone, Debug, Default)]
pub struct Unit {
    pub(super) context: Option<String>,
    pub(super) message: Message,
    pub(super) prev_context: Option<String>,
    pub(super) prev_message: Message,
    pub(super) flags: HashSet<String>,
    pub(super) notes: Vec<Note>,
    pub(super) locations: Vec<String>,
    pub(super) comments: Vec<Comment>,
    pub(super) state: State,
    pub(super) obsolete: bool,
}

impl Unit {
    /// Get the context string.
    pub fn context(&self) -> Option<&str> {
        self.context.as_ref().map(String::as_str)
    }

    /// Get the message string.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Get the previous context (in fuzzy units).
    pub fn prev_context(&self) -> Option<&str> {
        self.prev_context.as_ref().map(String::as_str)
    }

    /// Get the previous message (in fuzzy units).
    pub fn prev_message(&self) -> &Message {
        &self.prev_message
    }

    /// Get the flags
    pub fn flags(&self) -> &HashSet<String> {
        return &self.flags;
    }

    /// Get the notes/comments.
    pub fn notes(&self) -> &Vec<Note> {
        &self.notes
    }

    /// Get locations.
    pub fn locations(&self) -> &Vec<String> {
        &self.locations
    }

    /// Get custom comments.
    pub fn comments(&self) -> &Vec<Comment> {
        &self.comments
    }

    /// Get the state.
    pub fn state(&self) -> State {
        self.state
    }

    /// Returns whether the unit should be used in application.
    pub fn is_translated(&self) -> bool {
        self.state == State::Final
    }

    /// Returns whether the unit is obsolete.
    pub fn is_obsolete(&self) -> bool {
        self.obsolete
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Origin;

    impl Unit {
        pub(crate) fn for_tests_empty() -> Self {
            let mut res = Self::default();

            res.flags = (1..=3).map(|i| format!("empty-flag-{}", i)).collect();
            res.notes = (1..=4)
                .map(|i| Note::new(Origin::Translator, format!("empty translator note {}", i)))
                .collect();

            res.locations = vec![11, 22, 33]
                .into_iter()
                .enumerate()
                .map(|(i, n)| format!("EmptyFile{}:{}", i + 1, n))
                .collect();

            res.state = State::Empty;
            res
        }

        pub(crate) fn for_tests_normal() -> Self {
            let mut res = Self::default();

            res.prev_context = Some(String::from("prev-context"));
            res.prev_message = Message::Simple {
                id: String::from("prev-message"),
                text: Some(String::from("prev-text")),
            };

            res.context = Some(String::from("context"));
            res.message = Message::Simple {
                id: String::from("message"),
                text: Some(String::from("text")),
            };

            res.flags = (1..=3).map(|i| format!("flag{}", i)).collect();
            res.comments = (1..=3).map(|i| Comment::new('X', format!("Comment {}", i))).collect();
            res.notes = (1..=2)
                .map(|i| Note::new(Origin::Translator, format!("translator note {}", i)))
                .chain(
                    (1..=2)
                        .into_iter()
                        .map(|i| Note::new(Origin::Developer, format!("developper note {}", i))),
                )
                .collect();

            res.locations = vec![12, 34, 56]
                .into_iter()
                .enumerate()
                .map(|(i, n)| format!("File{}:{}", i + 1, n))
                .collect();

            res.state = State::Final;
            res
        }

        pub(crate) fn for_tests_incomplete() -> Self {
            let mut res = Self::default();

            res.context = Some(String::from("incomplete context"));
            res.message = Message::Simple {
                id: String::from("incomplete message"),
                text: Some(String::from("incomplete text")),
            };

            res.flags = (1..=3).map(|i| format!("flag{}", i)).collect();
            res.obsolete = true;
            res.state = State::NeedsWork;

            res
        }
    }

    impl PartialEq<Self> for Unit {
        fn eq(&self, other: &Self) -> bool {
            (self.context == other.context)
                && (self.prev_context == other.prev_context)
                && (self.message == other.message)
                && (self.prev_message == other.prev_message)
                && (self.flags == other.flags)
                && (self.notes == other.notes)
                && (self.locations == other.locations)
                && (self.comments == other.comments)
                && (self.state == other.state)
                && (self.obsolete == other.obsolete)
        }
    }

    impl Eq for Unit {}

    #[test]
    fn test_func_context() {
        let empty = Unit::for_tests_empty();
        let unit = Unit::for_tests_normal();

        assert_eq!(empty.context(), None);
        assert_eq!(unit.context(), Some("context"));
    }

    #[test]
    fn test_func_message() {
        let empty = Unit::for_tests_empty();
        let unit = Unit::for_tests_normal();
        let message = Message::Simple {
            id: String::from("message"),
            text: Some(String::from("text")),
        };

        assert_eq!(empty.message(), &Message::default());
        assert_eq!(unit.message(), &message);
    }

    #[test]
    fn test_func_prev_context() {
        let empty = Unit::for_tests_empty();
        let unit = Unit::for_tests_normal();

        assert_eq!(empty.prev_context(), None);
        assert_eq!(unit.prev_context(), Some("prev-context"));
    }

    #[test]
    fn test_func_prev_message() {
        let empty = Unit::for_tests_empty();
        let unit = Unit::for_tests_normal();
        let message = Message::Simple {
            id: String::from("prev-message"),
            text: Some(String::from("prev-text")),
        };

        assert_eq!(empty.prev_message(), &Message::default());
        assert_eq!(unit.prev_message(), &message);
    }

    #[test]
    fn test_func_flags() {
        let unit = Unit::for_tests_normal();
        let flags = unit.flags();

        assert!(Unit::default().flags().is_empty(), "Empty unit should have no flag");
        assert!(!flags.is_empty(), "Normal unit should have flags");
        assert_eq!(flags.len(), 3);
        assert!(flags.contains("flag1"));
    }

    #[test]
    fn test_func_notes() {
        let unit = Unit::for_tests_normal();
        let notes = unit.notes();

        assert!(Unit::default().notes().is_empty(), "Empty unit should have no note");
        assert!(!notes.is_empty(), "Normal unit should have notes");
        assert_eq!(notes.len(), 4);

        if let Some(note) = notes.into_iter().next() {
            assert_eq!(note.value(), "translator note 1");
        } else {
            panic!("Normal unit should hanve at least one note");
        }
    }

    #[test]
    fn test_func_locations() {
        let unit = Unit::for_tests_normal();
        let locations = unit.locations();

        assert!(Unit::default().flags().is_empty(), "Empty unit should have no location");
        assert!(!locations.is_empty(), "Normal unit should have locations");
        assert_eq!(locations.len(), 3);
        assert_eq!(
            locations,
            &vec![
                String::from("File1:12"),
                String::from("File2:34"),
                String::from("File3:56"),
            ]
        );
    }

    #[test]
    fn test_func_comments() {
        let unit = Unit::for_tests_normal();
        let comments = unit.comments();

        assert!(
            Unit::default().comments().is_empty(),
            "Empty unit should have no comment"
        );
        assert!(!comments.is_empty(), "Normal unit should have comments");
        assert_eq!(comments.len(), 3);
        assert_eq!(
            comments,
            &vec![
                Comment::new('X', String::from("Comment 1")),
                Comment::new('X', String::from("Comment 2")),
                Comment::new('X', String::from("Comment 3")),
            ]
        );
    }

    #[test]
    fn test_func_state() {
        let unit = Unit::for_tests_normal();

        assert_eq!(Unit::default().state(), State::Empty);
        assert_eq!(unit.state(), State::Final);
    }

    #[test]
    fn test_func_is_translated() {
        let unit = Unit::for_tests_normal();

        assert!(!Unit::default().is_translated(), "Empty unit should not be translated");
        assert!(unit.is_translated(), "Normal unit should be translated");
    }

    #[test]
    fn test_func_is_obsolete() {
        let normal_unit = Unit::for_tests_normal();
        let incomplete_unit = Unit::for_tests_incomplete();

        assert!(!normal_unit.is_obsolete(), "Normal unit should not be obsolete");
        assert!(incomplete_unit.is_obsolete(), "Incomplete unit should be obsolete");
    }

    #[test]
    fn test_trait_clone() {
        let unit = Unit::for_tests_normal();

        assert_eq!(unit.clone(), unit);
    }

    #[test]
    fn test_trait_default() {
        let unit = Unit::default();

        assert_eq!(unit.state, State::Empty);
        assert_eq!(unit.message, Message::default());
        assert_eq!(unit.prev_message, Message::default());
        assert_eq!(unit.context, None);
        assert_eq!(unit.prev_context, None);
        assert!(unit.flags.is_empty(), "For empty unit, there should be no flag");
        assert!(unit.notes.is_empty(), "For empty unit, there should be no note");
        assert!(unit.locations.is_empty(), "For empty unit, there should be no location");
        assert!(!unit.obsolete, "Empty unit should not be obsolete");
    }

    #[test]
    fn test_trait_debug() {
        assert_eq!(
            format!("{:?}", Unit::default()),
            String::from(
                "Unit { \
                    context: None, \
                    message: Simple { id: \"\", text: None }, \
                    prev_context: None, \
                    prev_message: Simple { id: \"\", text: None }, \
                    flags: {}, \
                    notes: [], \
                    locations: [], \
                    comments: [], \
                    state: Empty, \
                    obsolete: false \
                }"
            ),
        )
    }
}
// no-coverage:stop
