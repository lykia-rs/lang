use rustc_hash::FxHashMap;

use crate::{
    util::{alloc_shared, Shared},
    value::{
        callable::{Callable, CallableKind, Function},
        RV,
    },
};

use self::{
    fib::nt_fib,
    json::{nt_json_decode, nt_json_encode},
    out::nt_print,
    time::nt_clock,
};

use super::interpreter::Output;

pub mod fib;
pub mod json;
pub mod out;
pub mod time;

pub fn stdlib(out: Option<Shared<Output>>) -> FxHashMap<String, RV> {
    let mut std = FxHashMap::default();

    let mut benchmark_namespace = FxHashMap::default();
    let mut json_namespace = FxHashMap::default();
    let mut time_namespace = FxHashMap::default();
    let mut io_namespace = FxHashMap::default();

    benchmark_namespace.insert(
        "fib".to_owned(),
        RV::Callable(Callable::new(
            Some(1),
            CallableKind::Generic,
            Function::Lambda { function: nt_fib },
        )),
    );

    json_namespace.insert(
        "stringify".to_owned(),
        RV::Callable(Callable::new(
            Some(1),
            CallableKind::Generic,
            Function::Lambda {
                function: nt_json_encode,
            },
        )),
    );

    json_namespace.insert(
        "parse".to_owned(),
        RV::Callable(Callable::new(
            Some(1),
            CallableKind::Generic,
            Function::Lambda {
                function: nt_json_decode,
            },
        )),
    );

    time_namespace.insert(
        "clock".to_owned(),
        RV::Callable(Callable::new(
            Some(0),
            CallableKind::Generic,
            Function::Lambda { function: nt_clock },
        )),
    );

    io_namespace.insert(
        "print".to_owned(),
        RV::Callable(Callable::new(
            None,
            CallableKind::Generic,
            Function::Lambda { function: nt_print },
        )),
    );

    if out.is_some() {
        let mut test_namespace = FxHashMap::default();

        test_namespace.insert(
            "out".to_owned(),
            RV::Callable(Callable::new(
                None,
                CallableKind::Generic,
                Function::Stateful(out.unwrap().clone()),
            )),
        );

        std.insert(
            "test_utils".to_owned(),
            RV::Object(alloc_shared(test_namespace)),
        );
    }

    std.insert(
        "Benchmark".to_owned(),
        RV::Object(alloc_shared(benchmark_namespace)),
    );
    std.insert("json".to_owned(), RV::Object(alloc_shared(json_namespace)));
    std.insert("time".to_owned(), RV::Object(alloc_shared(time_namespace)));
    std.insert("io".to_owned(), RV::Object(alloc_shared(io_namespace)));

    std
}
