use std::time;
use crate::lang::execution::interpreter::Interpreter;
use crate::lang::execution::primitives::{HaltReason, RV};

pub fn nt_clock(_interpreter: &mut Interpreter, _args: &[RV]) -> Result<RV, HaltReason> {
    if let Ok(n) = time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        return Ok(RV::Num(n.as_secs_f64()));
    }
    Ok(RV::Undefined)
}