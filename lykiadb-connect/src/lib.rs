use tokio::net::TcpStream;
use crate::session::ClientSession;
use crate::tcp::TcpClientSession;

pub mod session;
mod tcp;

pub use lykiadb_server::net::{Message, Request, Response};
pub use lykiadb_server::runtime::error::{report_error};

pub enum Protocol {
    Tcp,
    Http
}

pub async fn get_session(addr: &str, protocol: Protocol) -> impl ClientSession {
    match protocol {
        Protocol::Tcp => {
            let socket = TcpStream::connect(addr).await.unwrap();
            TcpClientSession::new(socket)
        }
        Protocol::Http => {
            panic!("Http not implemented!")
        }
    }
}
pub async fn connect(addr: &str) -> impl ClientSession {
    get_session(addr, Protocol::Tcp).await
}