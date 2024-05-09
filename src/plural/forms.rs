use super::formula::Formula;
use crate::{error::Error, PoParser};

/// Decoded information from the header `Plural-Forms`
#[derive(Clone, Debug)]
pub struct PluralForms {
    formula: Formula,
    count: usize,
    definition: String,
    formula_source: String,
}

impl PluralForms {
    pub(crate) fn parse(input: &str, parser: &PoParser) -> Result<PluralForms, Error> {
        let values = parser.parse_map(input)?;
        let formula_source = values.get("plural").map(|s| s.to_string()).unwrap_or_default();
        let formula = Formula::parse(&formula_source)?;
        let count: usize = match values.get("nplurals") {
            None => 2,
            Some(s) => match s.parse() {
                Ok(v) => v,
                Err(err) => {
                    return Err(Error::PluralForms(err.to_string()));
                }
            },
        };

        Ok(PluralForms {
            formula,
            count,
            definition: input.to_string(),
            formula_source,
            // no-coverage:start
        })
        // no-coverage:stop
    }

    pub fn get_value(&self, count: usize) -> Option<usize> {
        self.formula.execute(count).filter(|v| *v < self.count)
    }

    pub fn get_count(&self) -> usize {
        self.count
    }

    pub fn get_definition(&self) -> &str {
        &self.definition
    }

    pub fn get_formula(&self) -> &str {
        &self.formula_source
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    impl PluralForms {
        pub(crate) fn for_tests_empty() -> PluralForms {
            PluralForms {
                formula: Formula::for_tests_empty(),
                count: 0,
                formula_source: String::new(),
                definition: String::new(),
            }
        }

        pub(crate) fn for_tests_shift() -> PluralForms {
            PluralForms {
                formula: Formula::for_tests_shift(),
                count: 200,
                formula_source: String::from("n-100"),
                definition: String::from("nplurals=200; plural=n-100"),
            }
        }

        pub(crate) fn for_tests_simple() -> PluralForms {
            make_forms(COUNT_CASE1, FORMULA_CASE1).0
        }
    }

    impl PartialEq for PluralForms {
        fn eq(&self, other: &Self) -> bool {
            (self.count == other.count) && (self.formula == other.formula)
        }
    }

    impl Eq for PluralForms {}

    const COUNT_CASE1: Option<usize> = Some(3);
    const FORMULA_CASE1: &str = "(n%10==1 && n%100!=11 ? 0 : n%10>=2 && (n%100<10 or n%100>=20) ? 1 : 2)";

    const COUNT_CASE2: Option<usize> = None;
    const FORMULA_CASE2: &str = "n>1 ? 0 : 1";

    fn make_forms(count: Option<usize>, formula: &str) -> (PluralForms, String) {
        let definition = match count {
            Some(count) => format!("nplurals={count}; plural={formula};"),
            None => format!("plural={formula};"),
        };

        let parser = PoParser::new();
        let res = PluralForms::parse(&definition, &parser).unwrap();

        (res, definition)
    }

    fn make_cases() -> Vec<(usize, usize)> {
        vec![
            (1, 0),
            (21, 0),
            (31, 0),
            (41, 0),
            (121, 0),
            (131, 0),
            (10, 2),
            (20, 2),
            (110, 2),
            (120, 2),
            (210, 2),
            (11, 2),
            (111, 2),
            (211, 2),
            (14, 2),
            (114, 2),
            (2, 1),
            (5, 1),
            (24, 1),
            (102, 1),
            (105, 1),
            (124, 1),
        ]
    }

    #[test]
    fn test_func_with_error() {
        let parser = PoParser::new();
        let res = PluralForms::parse("nplurals=abc; plural=n>1 ? 0 : 1;", &parser);

        assert!(
            res.is_err(),
            "The parser should return an error for parsing of `nplurals` {res:?}",
        );
    }

    #[test]
    fn test_func_get_value() {
        let forms = make_forms(COUNT_CASE1, FORMULA_CASE1).0;

        for (count, index) in make_cases() {
            assert_eq!(forms.get_value(count), Some(index), "For {}", count);
        }
    }

    #[test]
    fn test_func_get_count() {
        let forms = make_forms(COUNT_CASE1, FORMULA_CASE1).0;

        assert_eq!(forms.get_count(), 3);
    }

    #[test]
    fn test_func_get_count_with_default() {
        let forms = make_forms(COUNT_CASE2, FORMULA_CASE2).0;

        assert_eq!(forms.get_count(), 2);
    }

    #[test]
    fn test_func_get_definition() {
        let (forms, definition) = make_forms(COUNT_CASE1, FORMULA_CASE1);

        assert_eq!(forms.get_definition(), &definition);
    }

    #[test]
    fn test_func_get_formula() {
        let forms = make_forms(COUNT_CASE1, FORMULA_CASE1).0;

        assert_eq!(forms.get_formula(), FORMULA_CASE1);
    }

    #[test]
    fn test_forms() {
        let (forms, definition) = make_forms(COUNT_CASE1, FORMULA_CASE1);

        assert_eq!(&forms.formula_source, FORMULA_CASE1, "Formula");
        assert_eq!(forms.definition, definition, "Definition");
        assert_eq!(forms.count, 3);

        for (count, index) in make_cases() {
            assert_eq!(forms.formula.execute(count), Some(index), "For {}", count);
        }
    }

    #[test]
    fn test_struct() {
        let (forms, definition) = make_forms(COUNT_CASE1, FORMULA_CASE1);
        let copy = forms.clone();

        assert_eq!(copy.formula, forms.formula, "Formula was not cloned");
        assert_eq!(copy.count, forms.count, "Counts are not equals");
        assert_eq!(copy.definition, forms.definition, "Definitions differs");
        assert_eq!(copy.formula_source, forms.formula_source, "Formula sources differs");

        assert_eq!(
            format!("{:?}", copy),
            format!(
                "PluralForms {{ formula: {:?}, count: 3, definition: {:?}, formula_source: {:?} }}",
                forms.formula, definition, FORMULA_CASE1,
            ),
        );

        assert_eq!(copy, forms);
    }

    #[test]
    fn test_error_parse_on_nplurals() {
        let parser = PoParser::new();
        let res = PluralForms::parse("nplurals=wrong; plural=0", &parser);

        assert!(
            res.is_err(),
            "The parser should return an error for parsing of `nplurals`"
        );
    }
}
// no-coverage:stop
