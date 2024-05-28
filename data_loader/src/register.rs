use crate::configuration::*;

#[derive(Debug)]
pub struct Register {
    pub config: Configuration,
}

impl Register {
    pub fn new(config: Configuration) -> Self {
        Self { config }
    }
}
