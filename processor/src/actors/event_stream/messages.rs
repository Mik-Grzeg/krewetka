use actix::Message;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InitConsumer;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InitRetrier;
