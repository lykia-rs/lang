use lykiadb_server::net::{CommunicationError, Connection, Message, Request, Response};
use lykiadb_server::runtime::types::RV;
use lykiadb_server::runtime::{Runtime, RuntimeMode};
use std::io::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt as _;
use tracing::{error, info};

const ASCII_ART: &str = r"
$$\                 $$\       $$\           $$$$$$$\  $$$$$$$\
$$ |                $$ |      \__|          $$  __$$\ $$  __$$\
$$ |      $$\   $$\ $$ |  $$\ $$\  $$$$$$\  $$ |  $$ |$$ |  $$ |
$$ |      $$ |  $$ |$$ | $$  |$$ | \____$$\ $$ |  $$ |$$$$$$$\ |
$$ |      $$ |  $$ |$$$$$$  / $$ | $$$$$$$ |$$ |  $$ |$$  __$$\
$$ |      $$ |  $$ |$$  _$$<  $$ |$$  __$$ |$$ |  $$ |$$ |  $$ |
$$$$$$$$\ \$$$$$$$ |$$ | \$$\ $$ |\$$$$$$$ |$$$$$$$  |$$$$$$$  |
\________| \____$$ |\__|  \__|\__| \_______|\_______/ \_______/
          $$\   $$ |
          \$$$$$$  |
           \______/
";

struct Server {
    listener: Option<TcpListener>,
}

pub struct ServerSession {
    conn: Connection,
    runtime: Runtime,
}

impl ServerSession {
    pub fn new(stream: TcpStream) -> Self {
        ServerSession {
            conn: Connection::new(stream),
            runtime: Runtime::new(RuntimeMode::File),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            info!("{:?}", message);
            match message {
                Message::Request(req) => match req {
                    Request::Ast(source) => {
                        let ast = self.runtime.ast(&source);
                        self.conn
                            .write(Message::Response(Response::Program(
                                ast.unwrap(),
                            )))
                            .await
                            .unwrap();
                    }
                    Request::Run(command) => {
                        let execution = self.runtime.interpret(&command);
                        let response = if execution.is_ok() {
                            Response::Value(execution.ok().or_else(|| Some(RV::Undefined)).unwrap())
                        } else {
                            Response::Error(execution.err().unwrap())
                        };

                        self.conn.write(Message::Response(response)).await.unwrap();
                    }
                },
                _ => error!("Unsupported message type"),
            }
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}

impl Server {
    pub fn new() -> Result<Self, Error> {
        Ok(Server { listener: None })
    }

    pub async fn listen(mut self, addr: &str) -> Result<Self, Error> {
        let listener = TcpListener::bind(addr).await?;
        println!("{ASCII_ART}");
        info!("Listening on {}", listener.local_addr()?);
        self.listener = Some(listener);
        Ok(self)
    }

    pub async fn serve(self) -> Result<(), Error> {
        if let Some(listener) = self.listener {
            let mut stream = TcpListenerStream::new(listener);
            while let Some(socket) = stream.try_next().await? {
                let peer = socket.peer_addr()?;
                tokio::spawn(async move {
                    let mut session = ServerSession::new(socket);
                    info!("Client {} connected", peer);
                    session.handle().await;
                    info!("Client {} disconnected", peer);
                });
            }
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    Server::new()?.listen("0.0.0.0:19191").await?.serve().await
}
