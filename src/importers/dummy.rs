use tokio_stream::Stream;

struct DummyGenerator { }

impl DummyGenerator {
    fn run() -> impl Stream {
        todo!()
    }
}

impl Import for DummyGenerator {
    fn import(&self) -> Result<Vec<u8>, ImporterError> {
        todo!()
    }
}


