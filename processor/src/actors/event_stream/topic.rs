use async_trait::async_trait;

#[async_trait]
pub trait TopicOffsetKeeper {
    async fn inc_offset(&mut self, topic: String);
    fn stash_processed_offset(&mut self, offset: i64, retry: i32);
}
