use crate::environment::Environment;
use crate::object::Object;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn globals() -> Environment {
    let mut env = Environment::new();
    env.define("clock".to_string(), Object::Callable { arity: 0, func: clock });
    env
}

fn clock(_args: Vec<Object>) -> Object {
    let timestamp_f64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Object::Number(timestamp_f64)
}
