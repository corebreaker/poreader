#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum UnOp {
    Neg,
    Not,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[inline]
fn bool_to_num(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

#[inline]
fn get_infinity(v: i64) -> i64 {
    if v < 0 {
        i64::MIN
    } else {
        i64::MAX
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Node {
    Var,
    Num(i64),
    UnOp {
        op: UnOp,
        rhs: Box<Node>,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Cond {
        test: Box<Node>,
        if_true: Box<Node>,
        if_false: Box<Node>,
    },
}

impl Node {
    pub(super) fn new_num(v: i64) -> Node {
        Node::Num(v)
    }

    pub(super) fn new_unop(op: UnOp, rhs: Node) -> Node {
        Node::UnOp { op, rhs: Box::new(rhs) }
    }

    pub(super) fn new_binop(op: BinOp, lhs: Node, rhs: Node) -> Node {
        Node::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub(super) fn new_cond(test: Node, if_true: Node, if_false: Node) -> Node {
        Node::Cond {
            test: Box::new(test),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }

    pub(super) fn execute(&self, n: i64) -> i64 {
        match self {
            Node::Var => n,
            Node::Num(v) => *v,
            Node::UnOp { op, rhs } => match op {
                UnOp::Not => bool_to_num(rhs.execute(n) == 0),
                UnOp::Neg => -rhs.execute(n),
            },
            Node::BinOp { op, lhs, rhs } => {
                let lhs = lhs.execute(n);

                match op {
                    BinOp::Add => lhs.overflowing_add(rhs.execute(n)).0,
                    BinOp::Sub => lhs.overflowing_sub(rhs.execute(n)).0,
                    BinOp::Mul => lhs.overflowing_mul(rhs.execute(n)).0,
                    BinOp::Div => {
                        let rhs = rhs.execute(n);

                        if rhs != 0 {
                            lhs.overflowing_div(rhs).0
                        } else {
                            get_infinity(lhs)
                        }
                    }
                    BinOp::Mod => {
                        let rhs = rhs.execute(n);

                        if rhs != 0 {
                            lhs.overflowing_rem(rhs).0
                        } else {
                            lhs
                        }
                    }
                    BinOp::And => bool_to_num((lhs != 0) && (rhs.execute(n) != 0)),
                    BinOp::Or => bool_to_num((lhs != 0) || (rhs.execute(n) != 0)),
                    BinOp::Eq => bool_to_num(lhs == rhs.execute(n)),
                    BinOp::Ne => bool_to_num(lhs != rhs.execute(n)),
                    BinOp::Lt => bool_to_num(lhs < rhs.execute(n)),
                    BinOp::Lte => bool_to_num(lhs <= rhs.execute(n)),
                    BinOp::Gt => bool_to_num(lhs > rhs.execute(n)),
                    BinOp::Gte => bool_to_num(lhs >= rhs.execute(n)),
                }
            }
            Node::Cond {
                test,
                if_true,
                if_false,
            } => {
                if test.execute(n) != 0 {
                    if_true.execute(n)
                } else {
                    if_false.execute(n)
                }
            }
        }
    }
}

impl PartialEq<Self> for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Var, Node::Var) => true,
            (Node::Num(l), Node::Num(r)) => l == r,
            (Node::UnOp { op: lo, rhs: lr }, Node::UnOp { op: ro, rhs: rr }) => (lo == ro) && (lr == rr),
            (
                Node::BinOp {
                    op: lo,
                    lhs: ll,
                    rhs: lr,
                },
                Node::BinOp {
                    op: ro,
                    lhs: rl,
                    rhs: rr,
                },
            ) => (lo == ro) && (ll == rl) && (lr == rr),
            (
                Node::Cond {
                    test: lc,
                    if_true: lt,
                    if_false: lf,
                },
                Node::Cond {
                    test: rc,
                    if_true: rt,
                    if_false: rf,
                },
            ) => (lc == rc) && (lt == rt) && (lf == rf),
            _ => false,
        }
    }
}

impl Eq for Node {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestCase {
        test_name: &'static str,
        node: Node,
        exec_cases: HashMap<i64, i64>,
    }

    impl TestCase {
        fn make_tests() -> Vec<Self> {
            vec![
                TestCase {
                    test_name: "Variable",
                    node: Node::Var,
                    exec_cases: vec![(-100, -100), (-10, -10), (100, 100)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Constant",
                    node: Node::new_num(100),
                    exec_cases: vec![(-100, 100), (-10, 100), (0, 100), (5, 100), (100, 100)]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Operator `+`",
                    node: Node::new_binop(BinOp::Add, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-100, -90), (-10, 0), (100, 110)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `-`",
                    node: Node::new_binop(BinOp::Sub, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-100, -110), (5, -5), (10, 0), (100, 90)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `*`",
                    node: Node::new_binop(BinOp::Mul, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-2, -20), (0, 0), (5, 50)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `/`",
                    node: Node::new_binop(BinOp::Div, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-2, 0), (-20, -2), (0, 0), (20, 2), (35, 3)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `/` (inverse)",
                    node: Node::new_binop(BinOp::Div, Node::new_num(1000), Node::Var),
                    exec_cases: vec![(0, i64::MAX), (-10, -100), (100, 10)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `%`",
                    node: Node::new_binop(BinOp::Mod, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, -2), (-10, 0), (0, 0), (23, 3), (35, 5)]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Operator `==`",
                    node: Node::new_binop(BinOp::Eq, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 0), (2, 0), (100, 0), (10, 1)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `!=`",
                    node: Node::new_binop(BinOp::Ne, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 1), (2, 1), (100, 1), (10, 0)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `<`",
                    node: Node::new_binop(BinOp::Lt, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 1), (2, 1), (100, 0), (10, 0)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `<=`",
                    node: Node::new_binop(BinOp::Lte, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 1), (2, 1), (100, 0), (10, 1)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `>`",
                    node: Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 0), (2, 0), (100, 1), (10, 0)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `>=`",
                    node: Node::new_binop(BinOp::Gte, Node::Var, Node::new_num(10)),
                    exec_cases: vec![(-12, 0), (2, 0), (100, 1), (10, 1)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `!` (not)",
                    node: Node::new_unop(UnOp::Not, Node::Var),
                    exec_cases: vec![(-12, 0), (100, 0), (0, 1)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator 'neg'",
                    node: Node::new_unop(UnOp::Neg, Node::Var),
                    exec_cases: vec![(-12, 12), (100, -100), (0, 0)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Operator `&&`",
                    node: Node::new_binop(
                        BinOp::And,
                        Node::new_binop(BinOp::Lt, Node::new_num(-5), Node::Var),
                        Node::new_binop(BinOp::Lte, Node::Var, Node::new_num(25)),
                    ),
                    exec_cases: vec![(-12, 0), (100, 0), (0, 1), (-3, 1), (10, 1), (-5, 0), (25, 1)]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Operator `||`",
                    node: Node::new_binop(
                        BinOp::Or,
                        Node::new_binop(BinOp::Gte, Node::new_num(-5), Node::Var),
                        Node::new_binop(BinOp::Gt, Node::Var, Node::new_num(25)),
                    ),
                    exec_cases: vec![(-12, 1), (100, 1), (0, 0), (-3, 0), (10, 0), (-5, 1), (25, 0)]
                        .into_iter()
                        .collect(),
                },
                TestCase {
                    test_name: "Expression with `?`",
                    node: Node::new_cond(
                        Node::new_binop(BinOp::Lt, Node::Var, Node::new_num(10)),
                        Node::new_num(1),
                        Node::new_num(2),
                    ),
                    exec_cases: vec![(-12, 1), (100, 2), (0, 1), (-3, 1), (10, 2)].into_iter().collect(),
                },
                TestCase {
                    test_name: "Big expression",
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
                    exec_cases: vec![
                        (-12, -22),
                        (0, -10),
                        (10, 0),
                        (43, 10),
                        (53, 10),
                        (55, 20),
                        (44, 20),
                        (441, 1234),
                        (404, 1234),
                        (150, 850),
                        (156, 844),
                        (200, 800),
                    ]
                    .into_iter()
                    .collect(),
                },
            ]
        }

        fn run(&self) {
            let prefix = format!("For test {}", self.test_name);

            for (count, expected) in self.exec_cases.iter() {
                let res = self.node.execute(*count);

                assert_eq!(&res, expected, "{}, bad execution result for count {}", prefix, count);
            }
        }
    }

    #[test]
    fn test_enum_node() {
        assert_ne!(Node::Num(0).clone(), Node::Var.clone());
        assert_eq!(format!("{:?}", Node::Var), String::from("Var"));
        assert_eq!(format!("{:?}", Node::Num(0)), String::from("Num(0)"));

        let mut n: Node;

        n = Node::UnOp {
            op: UnOp::Not,
            rhs: Box::new(Node::Var),
        };
        assert_ne!(n.clone(), Node::Var);
        assert_eq!(format!("{:?}", n), String::from("UnOp { op: Not, rhs: Var }"));

        n = Node::BinOp {
            op: BinOp::Add,
            lhs: Box::new(Node::Var),
            rhs: Box::new(Node::Var),
        };
        assert_ne!(n.clone(), Node::Var);
        assert_eq!(
            format!("{:?}", n),
            String::from("BinOp { op: Add, lhs: Var, rhs: Var }")
        );

        n = Node::Cond {
            test: Box::new(Node::Var),
            if_true: Box::new(Node::Var),
            if_false: Box::new(Node::Var),
        };
        assert_ne!(n.clone(), Node::Var);
        assert_eq!(
            format!("{:?}", n),
            String::from("Cond { test: Var, if_true: Var, if_false: Var }")
        );
    }

    macro_rules! check_enum_variant {
        ($enum:ident, $variant:ident) => {
            assert_eq!($enum::$variant.clone(), $enum::$variant);
            assert_eq!(format!("{:?}", $enum::$variant), stringify!($variant));
        };
    }

    #[test]
    fn test_enum_unop() {
        check_enum_variant!(UnOp, Not);
        check_enum_variant!(UnOp, Neg);
    }

    #[test]
    fn test_enum_binop() {
        check_enum_variant!(BinOp, Add);
        check_enum_variant!(BinOp, Sub);
        check_enum_variant!(BinOp, Mul);
        check_enum_variant!(BinOp, Div);
        check_enum_variant!(BinOp, Mod);
        check_enum_variant!(BinOp, And);
        check_enum_variant!(BinOp, Or);
        check_enum_variant!(BinOp, Eq);
        check_enum_variant!(BinOp, Ne);
        check_enum_variant!(BinOp, Lt);
        check_enum_variant!(BinOp, Lte);
        check_enum_variant!(BinOp, Gt);
        check_enum_variant!(BinOp, Gte);
    }

    #[test]
    fn test_func_get_infinity() {
        assert_eq!(get_infinity(10), i64::MAX);
        assert_eq!(get_infinity(-10), i64::MIN);
    }

    #[test]
    fn test_func_bool_to_num() {
        assert_eq!(bool_to_num(false), 0);
        assert_eq!(bool_to_num(true), 1);
    }

    #[test]
    fn execute_nodes() {
        TestCase::make_tests().into_iter().for_each(|t| t.run());
    }
}
