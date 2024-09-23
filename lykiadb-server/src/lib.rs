pub mod comm;
pub mod engine;
pub mod plan;
pub mod util;
pub mod value;

#[macro_export]
macro_rules! assert_plan {
    ($($name:ident: {$field:literal => $value:literal}),*) => {
        $(
            #[test]
            fn $name() {
                expect_plan($field, $value);
            }
        )*
    };
}
