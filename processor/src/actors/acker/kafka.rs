use crate::actors::event_stream::kafka::KafkaState;

#[async_trait]
impl Acknowleger for KafkaState {
    async fn end_processing(&self, _id: i64) {
        todo!()
    }
}

impl Acknowleger for KafkaState {

}
