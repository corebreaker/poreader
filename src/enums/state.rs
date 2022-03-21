/// Translation state.
///
/// Indicates whether the translation is considered usable.
///
/// # TODO:
/// - Rejected, Unreviewed, NeedsReview (from TT), possibly more (note: obsolete is a separate flag)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// The unit is not translated.
    Empty,
    /// The unit is a suggestion that might be embarrassingly wrong, possibly automatic. It needs
    /// checking by human translator before it can be used. (Used for `#,fuzzy` entries in `.po`.)
    NeedsWork,
    /// The unit is considered usable.
    Final,
}

impl Default for State {
    fn default() -> State {
        State::Empty
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_enum() {
        assert_eq!(State::Empty.clone(), State::Empty);
        assert_eq!(State::NeedsWork.clone(), State::NeedsWork);
        assert_eq!(State::Final.clone(), State::Final);
    }

    #[test]
    fn test_default() {
        assert_eq!(State::default(), State::Empty);
        assert_eq!(format!("{:?}", State::default()), String::from("Empty"));
    }

    #[test]
    fn test_hash() {
        let m = {
            let mut m = HashMap::new();

            m.insert(State::Empty, 123);
            m
        };

        assert_eq!(m.get(&State::Final), None);
        assert_eq!(m.get(&State::Empty), Some(&123));
    }
}
// no-coverage:stop
