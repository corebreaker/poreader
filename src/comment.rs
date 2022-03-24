#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Comment {
    kind: char,
    content: String,
}

impl Comment {
    pub(super) fn new(kind: char, content: String) -> Comment {
        Comment { kind, content }
    }

    pub fn kind(&self) -> char {
        self.kind
    }

    pub fn comment(&self) -> &str {
        &self.content
    }
}

// no-coverage:start
#[cfg(test)]
mod tests {
    use super::*;

    fn make_comment() -> Comment {
        Comment::new('X', String::from("Comment"))
    }

    #[test]
    fn test_func_new() {
        assert_eq!(
            make_comment(),
            Comment {
                kind: 'X',
                content: String::from("Comment")
            },
        );
    }

    #[test]
    fn test_func_kind() {
        let comment = make_comment();

        assert_eq!(comment.kind(), 'X');
    }

    #[test]
    fn test_func_content() {
        let comment = make_comment();

        assert_eq!(comment.comment(), "Comment");
    }
}
// no-coverage:stop
