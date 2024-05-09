#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PoLine {
    Blank,

    // (line number, kind (translator is space), content of the comment)
    Comment(usize, char, String),

    // (line number, obsolete/previous flag, tag, string)
    Message(usize, String, String, String),

    // (line number, obsolete/previous flag, string)
    Continuation(usize, String, String),
}

impl PoLine {
    pub(super) fn line(&self) -> usize {
        match self {
            PoLine::Blank => 0,
            PoLine::Comment(l, ..) => *l,
            PoLine::Message(l, ..) => *l,
            PoLine::Continuation(l, ..) => *l,
        }
    }
}

impl Default for PoLine {
    fn default() -> Self {
        Self::Blank
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    impl PoLine {
        fn new_blank() -> Self {
            PoLine::Blank
        }

        fn new_comment() -> Self {
            PoLine::Comment(1, 'A', String::from("S0"))
        }

        fn new_message() -> Self {
            PoLine::Message(2, String::from("F1"), String::from("T"), String::from("S1"))
        }

        fn new_continuation() -> Self {
            PoLine::Continuation(3, String::from("F2"), String::from("S2"))
        }
    }

    #[test]
    fn test_enum() {
        assert_eq!(PoLine::new_comment().clone(), PoLine::new_comment());
        assert_eq!(PoLine::new_message().clone(), PoLine::new_message());
        assert_eq!(PoLine::new_continuation().clone(), PoLine::new_continuation());
        assert_eq!(PoLine::new_blank().clone(), PoLine::new_blank());
    }

    #[test]
    fn test_default() {
        assert_eq!(PoLine::default(), PoLine::Blank);
        assert_eq!(format!("{:?}", PoLine::default()), String::from("Blank"));
    }

    #[test]
    fn test_func_line() {
        assert_eq!(PoLine::new_blank().line(), 0);
        assert_eq!(PoLine::new_comment().line(), 1);
        assert_eq!(PoLine::new_message().line(), 2);
        assert_eq!(PoLine::new_continuation().line(), 3);
    }
}
// no-coverage:stop
