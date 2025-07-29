//use color_eyre::eyre;
//use tokio::net::TcpStream;
//use tracing::*;
//
//use crate::agents::dog::Reply;
//use crate::event::Event;
//use futures::{SinkExt, StreamExt};
//use tokio::sync::mpsc::Receiver;
//use tokio::sync::mpsc::UnboundedSender;
////use tungstenite::protocal::Message;
//
//pub async fn handle_socket(
//    port: usize,
//    socket: TcpStream,
//    event_sender: UnboundedSender<Event>,
//    rx: &mut Receiver<Reply>,
//) -> eyre::Result<()> {
//    info!("hello socket");
//    let ws_stream = tokio_tungstenite::accept_async(socket).await?;
//    info!("new ws conn");
//
//    let (mut write, mut read) = ws_stream.split();
//    loop {
//        if let Some(msg) = read.next().await {
//            let msg = msg?.to_string();
//            info!("rec: {msg}");
//            event_sender.send(Event::AgentCommand(port, msg))?;
//        };
//        match rx.try_recv() {
//            Ok(reply) => write.send(reply.to_string().into()).await?,
//            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => (),
//            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => panic!("oh no!"),
//        }
//    }
//}
