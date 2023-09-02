use std::rc::Rc;
use serde_json::json;

use crate::lang::execution::interpreter::Interpreter;
use crate::lang::parsing::types::CallableError;
use crate::lang::parsing::types::RV;

pub fn nt_json_encode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, CallableError> {
    return Ok(RV::Str(Rc::new(json!(args[0]).to_string())));
}

pub fn nt_json_decode(_interpreter: &mut Interpreter, args: &[RV]) -> Result<RV, CallableError> {
    let json_str = match &args[0] {
        RV::Str(s) => s,
        _ => return Err(CallableError::GenericError("json_decode: expected string".to_string()))
    };

    let parsed: RV = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => return Err(CallableError::GenericError(format!("json_decode: {}", e)))
    };

    Ok(parsed)
}