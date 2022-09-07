use super::errors::ExporterError;
use std::fmt::Debug;
use std::future::Future;

pub trait Export {

    fn export(&self, message: Vec<u8>) -> Box<dyn Future<Output = Result<(), ExporterError>>>;
    fn settings(&self) -> String;
}

impl Debug for dyn Export {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Exporter: {{{}}}", self.settings())
    }
}