use lykiadb_server::net::Message;

pub trait ClientSession {
    async fn send_receive(&mut self, msg: Message) -> Result<Message, ()>;
}