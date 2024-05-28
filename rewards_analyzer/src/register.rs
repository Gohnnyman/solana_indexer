use crate::configuration::*;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Register {
    pub configuration: Configuration,
}
impl Register {
    pub fn current() -> Arc<Register> {
        CURRENT_REGISTER.with(|c| c.read().unwrap().clone())
    }
}

thread_local! {
    static CURRENT_REGISTER:RwLock<Arc<Register>> = RwLock::new(Arc::new(Register { configuration: Configuration::new().unwrap() }))
}
