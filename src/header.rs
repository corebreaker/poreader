/// Contrainer of an header entry
///
/// It contains the name and the value
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Header { name, value }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    fn make_test() -> Header {
        Header {
            name: String::from("Name"),
            value: String::from("Value"),
        }
    }

    #[test]
    fn test_func_new() {
        let header = Header::new(String::from("Name"), String::from("Value"));

        assert_eq!(make_test(), header);
        assert_eq!(header.name, "Name");
        assert_eq!(header.value, "Value");
    }

    #[test]
    fn test_func_clone() {
        let header = make_test();

        assert_eq!(header.clone(), header);
    }

    #[test]
    fn test_func_name() {
        let header = make_test();

        assert_eq!(header.name(), "Name");
    }

    #[test]
    fn test_func_value() {
        let header = make_test();

        assert_eq!(header.value(), "Value");
    }
}
// no-coverage:stop
