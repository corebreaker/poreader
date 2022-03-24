use super::PluralForms;
use std::rc::Rc;

/// Plural set
#[derive(Clone, Debug)]
pub struct Plural {
    forms: Option<Rc<PluralForms>>,
    singular: String,
    plural: String,
    values: Vec<String>,
}

impl Plural {
    pub(crate) fn new(singular: String, plural: String, values: Vec<String>, forms: Option<Rc<PluralForms>>) -> Self {
        Self {
            forms,
            singular,
            plural,
            values,
        }
    }

    pub fn singular(&self) -> &str {
        &self.singular
    }

    pub fn plural(&self) -> &str {
        &self.plural
    }

    pub fn first(&self) -> &str {
        self.values.iter().next().map(|s| s.as_str()).unwrap_or_default()
    }

    pub fn get(&self, count: usize) -> Option<&str> {
        self.forms.as_ref().and_then(|forms| {
            forms
                .get_value(count)
                .and_then(|index| self.values.get(index))
                .map(|v| v.as_str())
        })
    }

    pub fn values(&self) -> &Vec<String> {
        &self.values
    }

    pub fn is_blank(&self) -> bool {
        self.values.iter().all(String::is_empty)
    }

    pub fn get_forms(&self) -> Option<&PluralForms> {
        self.forms.as_ref().map(|f| f.as_ref())
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;
    use crate::PoParser;

    impl Plural {
        pub(crate) fn new_empty() -> Plural {
            Plural::new(String::new(), String::new(), vec![], None)
        }

        pub(crate) fn with_no_forms(singular: String, plural: Option<String>) -> Plural {
            Plural::new(singular, plural.unwrap_or_default(), vec![], None)
        }
    }

    impl PartialEq for Plural {
        fn eq(&self, other: &Self) -> bool {
            let (l, r) = (self, other);

            (l.singular == r.singular) && (l.plural == r.plural) && (l.values == r.values) && (l.forms == r.forms)
        }
    }

    impl Eq for Plural {}

    const SINGULAR_EN: &str = "My car is wonderful";
    const PLURAL_EN: &str = "My cars are wonderful";
    const SINGULAR_FR: &str = "Ma voiture est merveilleuse";
    const PLURAL_FR: &str = "Mes voitures sont merveilleuses";

    fn make_plural() -> Plural {
        let parser = PoParser::new();
        let forms = PluralForms::parse("nplurals=2; plural=n>1;", &parser).unwrap();

        Plural::new(
            String::from(SINGULAR_EN),
            String::from(PLURAL_EN),
            vec![String::from(SINGULAR_FR), String::from(PLURAL_FR)],
            Some(Rc::new(forms)),
        )
    }

    fn make_blank(values: Vec<String>) -> Plural {
        Plural::new(String::new(), String::new(), values, None)
    }

    #[test]
    fn test_func_new() {
        let plural = make_plural().clone();

        assert_eq!(plural.singular, String::from(SINGULAR_EN));
        assert_eq!(plural.plural, String::from(PLURAL_EN));
        assert_eq!(plural.values, vec![String::from(SINGULAR_FR), String::from(PLURAL_FR)]);
        assert!(plural.forms.is_some(), "Form should be a `Some`");
        assert_eq!(plural.forms.as_ref().map(|v| v.get_count()), Some(2));
        assert_eq!(
            plural.forms.as_ref().map(|v| v.get_formula()),
            Some("n>1")
        );

        assert_eq!(
            format!("{:?}", plural),
            format!(
                "Plural {{ forms: {:?}, singular: {:?}, plural: {:?}, values: {:?} }}",
                plural.forms, plural.singular, plural.plural, plural.values,
            )
        )
    }

    #[test]
    fn test_func_singular() {
        let plural = make_plural();

        assert_eq!(plural.singular(), SINGULAR_EN);
    }

    #[test]
    fn test_func_plural() {
        let plural = make_plural();

        assert_eq!(plural.plural(), PLURAL_EN);
    }

    #[test]
    fn test_func_first() {
        let plural = make_plural();

        assert_eq!(plural.first(), SINGULAR_FR);
        assert_eq!(make_blank(vec![]).first(), "");
    }

    #[test]
    fn test_func_values() {
        let plural = make_plural();
        let values = vec![SINGULAR_FR, PLURAL_FR];

        assert_eq!(plural.values(), &values);
    }

    #[test]
    fn test_func_is_blank() {
        let plural = make_plural();
        let empty_values = vec![String::new(), String::new()];
        let some_values = vec![String::new(), String::from("Something"), String::from("")];

        assert!(!plural.is_blank(), "This should not be blank");
        assert!(make_blank(vec![]).is_blank(), "With empty list, this should be blank");
        assert!(
            make_blank(empty_values).is_blank(),
            "With a list with all empty strings, this should be blank"
        );
        assert!(
            !make_blank(some_values).is_blank(),
            "With a list with some empty strings, this should not be blank"
        );
    }

    #[test]
    fn test_func_get_forms() {
        let blank = make_blank(vec![]);
        let plural = make_plural();
        let forms = plural.get_forms();

        assert!(blank.get_forms().is_none(), "Form should be a `None`");
        assert!(forms.is_some(), "Form should be a `Some`");

        if let Some(forms) = forms {
            assert_eq!(forms.get_count(), 2);
            assert_eq!(forms.get_formula(), "n>1");
        }
    }

    #[test]
    fn test_func_get() {
        let plural = make_plural();

        assert_eq!(plural.get(0), Some(SINGULAR_FR));
        assert_eq!(plural.get(1), Some(SINGULAR_FR));
        assert_eq!(plural.get(2), Some(PLURAL_FR));
        assert_eq!(plural.get(10), Some(PLURAL_FR));
        assert_eq!(plural.get(100), Some(PLURAL_FR));
    }
}
// no-coverage:stop
