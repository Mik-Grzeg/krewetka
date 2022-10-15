



use async_trait::async_trait;

#[async_trait]
pub trait TopicOffsetKeeper {
    async fn inc_offset(&mut self);
    fn stash_processed_offset(&mut self, offset: i64);
}
