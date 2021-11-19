mod node;

use crate::error::Error;
use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(formula, "/plural/formula/formula.rs");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Formula {
    expr: node::Node,
}

impl Formula {
    pub(super) fn parse(input: &str) -> Result<Self, Error> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(Formula { expr: node::Node::Var });
        }

        let parser = formula::FormulaParser::new();
        let res: Result<node::Node, ParseError<_, _, _>> = parser.parse(input);

        match res {
            Ok(expr) => Ok(Formula { expr }),
            Err(err) => Err(Error::PluralForms(err.to_string())),
        }
    }

    pub(super) fn execute(&self, count: usize) -> Option<usize> {
        let res = self.expr.execute(count as i64);

        if res < 0 {
            None
        } else {
            Some(res as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        node::{BinOp, Node, UnOp},
        Formula,
    };
    use std::collections::HashMap;

    struct TestCase {
        test_name: &'static str,
        source: &'static str,
        has_error: bool,
        node: Node,
        count_tests: HashMap<usize, Option<usize>>,
    }

    impl TestCase {
        fn make_tests() -> Vec<Self> {
            vec![
                TestCase {
                    test_name: "Empty string",
                    source: "",
                    has_error: false,
                    node: Node::Var,
                    count_tests: vec![(100, Some(100))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Variable",
                    source: "n",
                    has_error: false,
                    node: Node::Var,
                    count_tests: vec![(100, Some(100))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Constant",
                    source: "100",
                    has_error: false,
                    node: Node::new_num(100),
                    count_tests: vec![(0, Some(100)), (5, Some(100)), (100, Some(100))]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Zero",
                    source: "0",
                    has_error: false,
                    node: Node::new_num(0),
                    count_tests: vec![(0, Some(0)), (5, Some(0)), (100, Some(0))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Unrecognized",
                    source: "azerty",
                    has_error: true,
                    node: Node::Var,
                    count_tests: HashMap::new(),
                },
                TestCase {
                    test_name: "Operator `+`",
                    source: "n + 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Add, Node::Var, Node::new_num(10)),
                    count_tests: vec![(100, Some(110))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `+` (with error)",
                    source: "n + ",
                    has_error: true,
                    node: Node::Var,
                    count_tests: HashMap::new(),
                },
                TestCase {
                    test_name: "Operator `-`",
                    source: "n - 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Sub, Node::Var, Node::new_num(10)),
                    count_tests: vec![(5, None), (10, Some(0)), (100, Some(90))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `*`",
                    source: "n * 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Mul, Node::Var, Node::new_num(10)),
                    count_tests: vec![(0, Some(0)), (5, Some(50))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `/`",
                    source: "n / 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Div, Node::Var, Node::new_num(10)),
                    count_tests: vec![(0, Some(0)), (20, Some(2)), (35, Some(3))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `%`",
                    source: "n % 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Mod, Node::Var, Node::new_num(10)),
                    count_tests: vec![(0, Some(0)), (23, Some(3)), (35, Some(5))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `==`",
                    source: "n == 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Eq, Node::Var, Node::new_num(10)),
                    count_tests: vec![(100, Some(0)), (10, Some(1))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `!=`",
                    source: "n != 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Ne, Node::Var, Node::new_num(10)),
                    count_tests: vec![(2, Some(1)), (100, Some(1)), (10, Some(0))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `<`",
                    source: "n < 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Lt, Node::Var, Node::new_num(10)),
                    count_tests: vec![(2, Some(1)), (100, Some(0)), (10, Some(0))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `<=`",
                    source: "n <= 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Lte, Node::Var, Node::new_num(10)),
                    count_tests: vec![(2, Some(1)), (100, Some(0)), (10, Some(1))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `>`",
                    source: "n > 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(10)),
                    count_tests: vec![(2, Some(0)), (100, Some(1)), (10, Some(0))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `>=`",
                    source: "n >= 10",
                    has_error: false,
                    node: Node::new_binop(BinOp::Gte, Node::Var, Node::new_num(10)),
                    count_tests: vec![(2, Some(0)), (100, Some(1)), (10, Some(1))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `!` (not)",
                    source: "!n",
                    has_error: false,
                    node: Node::new_unop(UnOp::Not, Node::Var),
                    count_tests: vec![(100, Some(0)), (0, Some(1))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator 'neg'",
                    source: "-n",
                    has_error: false,
                    node: Node::new_unop(UnOp::Neg, Node::Var),
                    count_tests: vec![(100, None), (0, Some(0))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `&&`",
                    source: "(5 < n) && n <= 25",
                    has_error: false,
                    node: Node::new_binop(
                        BinOp::And,
                        Node::new_binop(BinOp::Lt, Node::new_num(5), Node::Var),
                        Node::new_binop(BinOp::Lte, Node::Var, Node::new_num(25)),
                    ),
                    count_tests: vec![(100, Some(0)), (0, Some(0)), (5, Some(0)), (10, Some(1)), (25, Some(1))]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Operator `||`",
                    source: "(5 >= n) || n > 25",
                    has_error: false,
                    node: Node::new_binop(
                        BinOp::Or,
                        Node::new_binop(BinOp::Gte, Node::new_num(5), Node::Var),
                        Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(25)),
                    ),
                    count_tests: vec![(100, Some(1)), (0, Some(1)), (5, Some(1)), (10, Some(0)), (25, Some(0))]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Expression with `?`",
                    source: "n < 10 ? 1 : 2",
                    has_error: false,
                    node: Node::new_cond(
                        Node::new_binop(BinOp::Lt, Node::Var, Node::new_num(10)),
                        Node::new_num(1),
                        Node::new_num(2),
                    ),
                    count_tests: vec![(100, Some(2)), (0, Some(1)), (10, Some(2))].into_iter().collect(),
                },
                TestCase {
                    test_name: "Big expression",
                    source: "n > 10 ? (n % 10) == 3 ? 10 : n < 100 ? 20 : (!(n > 200) ? -n + 1000 : 1234) : n - 10",
                    has_error: false,
                    node: Node::new_cond(
                        Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(10)),
                        Node::new_cond(
                            Node::new_binop(
                                BinOp::Eq,
                                Node::new_binop(BinOp::Mod, Node::Var, Node::new_num(10)),
                                Node::new_num(3),
                            ),
                            Node::new_num(10),
                            Node::new_cond(
                                Node::new_binop(BinOp::Lt, Node::Var, Node::new_num(100)),
                                Node::new_num(20),
                                Node::new_cond(
                                    Node::new_unop(
                                        UnOp::Not,
                                        Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(200)),
                                    ),
                                    Node::new_binop(
                                        BinOp::Add,
                                        Node::new_unop(UnOp::Neg, Node::Var),
                                        Node::new_num(1000),
                                    ),
                                    Node::new_num(1234),
                                ),
                            ),
                        ),
                        Node::new_binop(BinOp::Sub, Node::Var, Node::new_num(10)),
                    ),
                    count_tests: vec![
                        (0, None),
                        (10, Some(0)),
                        (43, Some(10)),
                        (53, Some(10)),
                        (55, Some(20)),
                        (44, Some(20)),
                        (441, Some(1234)),
                        (404, Some(1234)),
                        (150, Some(850)),
                        (156, Some(844)),
                        (200, Some(800)),
                    ]
                    .into_iter()
                    .collect(),
                },
                TestCase {
                    test_name: "Big expression (with error)",
                    source: "n > 10 ? (n % 10) == 3 ? 10 : (n < 100 ? 20 : (!(n > 200) ? -n + 1000 : 1234) : n - 10",
                    has_error: true,
                    node: Node::Var,
                    count_tests: HashMap::new(),
                },
            ]
        }

        fn run(&self) {
            let prefix = format!("For test {}", self.test_name);
            let input = self.source;
            let formula = match Formula::parse(input) {
                Err(err) => {
                    assert!(
                        self.has_error,
                        "{}, error found while parsig formula: `{}`: {:?}",
                        prefix, input, err
                    );

                    return;
                }
                Ok(formula) => {
                    assert!(
                        !self.has_error,
                        "{}, parser should return an error for source: `{}`",
                        prefix, input
                    );

                    formula
                }
            };

            assert_eq!(formula.expr, self.node, "{}", prefix);

            for (count, index) in self.count_tests.iter() {
                let res = formula.execute(*count);

                assert_eq!(
                    res.as_ref(),
                    index.as_ref(),
                    "{}, bad index for count {}",
                    prefix,
                    count
                )
            }
        }
    }

    #[test]
    fn formula() {
        TestCase::make_tests().into_iter().for_each(|t| t.run());
    }

    #[test]
    fn test_struct_formula() {
        let formula = Formula { expr: Node::Var };
        let copy = formula.clone();

        assert_eq!(copy.expr, formula.expr);
        assert_eq!(copy, formula);
        assert_eq!(format!("{:?}", formula), String::from("Formula { expr: Var }"));
    }

    impl Formula {
        pub(in super::super) fn for_tests_empty() -> Formula {
            Formula { expr: Node::new_num(0) }
        }

        pub(in super::super) fn for_tests_shift() -> Formula {
            Formula {
                expr: Node::new_binop(BinOp::Sub, Node::Var, Node::new_num(100)),
            }
        }
    }
}
