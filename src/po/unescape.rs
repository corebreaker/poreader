use regex::{Captures, Regex};

pub(super) struct Unescaper {
    re: Regex,
    table: [Option<&'static str>; 42],
}

impl Unescaper {
    #[rustfmt::skip]
    pub(super) fn new() -> Unescaper {
        Unescaper {
            re: Regex::new(r#"\\([rtn"\\])"#).unwrap(),
            table: [
                Some("\""), None,       None, None,       None, None,       None,
                None,       None,       None, None,       None, None,       None,
                None,       None,       None, None,       None, None,       None,
                None,       None,       None, None,       None, None,       None,
                None,       Some("\\"), None, None,       None, None,       None,
                None,       None,       None, Some("\n"), None, Some("\r"), Some("\t"),
            ]
        }
    }

    fn replace_char(&self, ch: char) -> Option<&'static str> {
        let idx = ch as u32;

        if ((idx & 1) == 0) && (34 <= idx) && (idx <= 116) {
            let index = ((idx - 34) / 2) as usize;

            self.table[index]
        } else {
            None
        }
    }

    pub(super) fn unescape(&self, text: &str) -> String {
        self.re
            .replace_all(text, |d: &Captures| -> String {
                d.get(1)
                    .map(|m| m.as_str())
                    .map(|key| key.chars().next().and_then(|ch| self.replace_char(ch)).unwrap_or(key))
                    .unwrap_or_default()
                    .to_string()
            })
            .to_string()
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_replace_char() {
        let unesc = Unescaper::new();

        assert_eq!(unesc.replace_char('A'), None);
        assert_eq!(unesc.replace_char('B'), None);
        assert_eq!(unesc.replace_char('C'), None);
        assert_eq!(unesc.replace_char('D'), None);
        assert_eq!(unesc.replace_char('E'), None);
        assert_eq!(unesc.replace_char('"'), Some("\""));
        assert_eq!(unesc.replace_char('\\'), Some("\\"));
        assert_eq!(unesc.replace_char('n'), Some("\n"));
        assert_eq!(unesc.replace_char('r'), Some("\r"));
        assert_eq!(unesc.replace_char('t'), Some("\t"));
    }

    #[test]
    fn test_func_unescape() {
        let unesc = Unescaper::new();

        assert_eq!(
            unesc.unescape(r"Hello\nworld\r\n\t!"),
            String::from("Hello\nworld\r\n\t!")
        );
        assert_eq!(unesc.unescape(r#"Sub\"\tstring"#), String::from("Sub\"\tstring"));
        assert_eq!(unesc.unescape(r"My\\Path: \tValue"), String::from("My\\Path: \tValue"));
    }
}
// no-coverage:stop
