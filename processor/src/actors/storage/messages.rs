use actix::Message;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InitFlusher;
