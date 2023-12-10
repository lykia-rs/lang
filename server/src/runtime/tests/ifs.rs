#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::runtime::{tests::helpers::exec_assert, types::RV};

    #[test]
    fn test_if() {
        exec_assert(
            "var $a = 30;

        if ($a > 50) {
            TestUtils.out(\"> 50\");
        }
        else if ($a > 20) {
            TestUtils.out(\"50 > $a > 20\");
        }
        else {
            TestUtils.out(\"< 20\");
        }",
            vec![RV::Str(Rc::new("50 > $a > 20".to_string()))],
        );
    }
}
