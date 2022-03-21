/// Note (comment) origins.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Origin {
    /// Comment from developer.
    Developer,

    /// Comment from translator.
    Translator,
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_enum() {
        assert_eq!(Origin::Developer.clone(), Origin::Developer);
        assert_eq!(Origin::Translator.clone(), Origin::Translator);
        assert_eq!(format!("{:?}", Origin::Developer), String::from("Developer"));
    }

    #[test]
    fn test_hash() {
        let m = {
            let mut m = HashMap::new();

            m.insert(Origin::Developer, 123);
            m
        };

        assert_eq!(m.get(&Origin::Translator), None);
        assert_eq!(m.get(&Origin::Developer), Some(&123));
    }
}
// no-coverage:stop
