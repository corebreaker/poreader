use super::Origin;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Note {
    origin: Origin,
    value: String,
}

impl Note {
    pub fn new(origin: Origin, value: String) -> Note {
        Note { origin, value }
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALUE: &str = "message";

    fn make_note() -> Note {
        Note::new(Origin::Translator, String::from(VALUE))
    }

    #[test]
    fn test_struct() {
        let note = make_note();

        assert_eq!(note.clone(), note);
        assert_eq!(
            format!("{:?}", note),
            format!("Note {{ origin: {:?}, value: {:?} }}", note.origin, note.value),
        );
    }

    #[test]
    fn test_func_origin() {
        let note = make_note();

        assert_eq!(note.origin(), &Origin::Translator);
    }

    #[test]
    fn test_func_value() {
        let note = make_note();

        assert_eq!(note.value(), VALUE);
    }
}
