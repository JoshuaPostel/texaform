use crate::event::Event;
use color_eyre::eyre;
use std::fmt::Display;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::TryRecvError;
use bytes::BytesMut;

use async_recursion::async_recursion;
use tracing::*;

// TODO update to protocall where all messages end in `;`
pub async fn handle_socket_old<R: Display + Send>(
    port: usize,
    mut socket: TcpStream,
    event_sender: UnboundedSender<Event>,
    rx: &mut Receiver<R>,
) -> eyre::Result<String> {
    info!("hello socket");
    let mut data = [0; 32];
    let mut heartbeat = 5;
    loop {
        info!("top of loop");
        let n = socket.read(&mut data).await?;
        //let n = io::read_until(socket, 59, &mut data).await?;
        info!("post read");
        if n == 0 {
            info!("n == 0 heartbeat: {heartbeat}");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            if heartbeat < 0 {
                return Ok("heartbeat timeout".to_string());
            } else {
                heartbeat -= 1;
            }
        } else {
            let msg = str::from_utf8(&data[0..n])?;
            info!("msg: {msg}");
            let reply = Event::AgentCommand(port, msg.to_string());
            info!("agent reply: {reply:?}");
            event_sender.send(reply)?;
            info!("post event_sender send");
            // TODO 100 here is the AGENT_UPDATE_MILLIS, which should be configured at top level
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            info!("post sleep");
            let reply = retry_recv(rx, 0).await;
            socket.write_all(reply.as_bytes()).await?;
            info!("post socket write_all");
            data = [0; 32];
        }
    }
}

// if for some reason the agent takes > 100 ms to reply, we retry
// initailly happened when saving/loading game
// moving more/all long blocking calls to a background task should remove the need for this
#[async_recursion]
async fn retry_recv<R: Display + Send>(rx: &mut Receiver<R>, try_count: u8) -> String {
    match rx.try_recv() {
        Ok(reply) => {
            info!("reply: {reply}");
            reply.to_string() + ";"
        }
        Err(TryRecvError::Empty) => {
            error!("TryRecvError {try_count}");
            // TODO excessive sleep time, test/profile lower sleep & higher try_count
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            if try_count > 10 {
                panic!("retry_recv failed multiple times")
            }
            //"f".to_string()
            retry_recv(rx, try_count + 1).await
        }
        Err(TryRecvError::Disconnected) => panic!("oh no!"),
    }
}

pub async fn handle_socket<R: Display + Send>(
    port: usize,
    mut socket: TcpStream,
    event_sender: UnboundedSender<Event>,
    rx: &mut Receiver<R>,
) -> eyre::Result<String> {
    info!("hello socket");
    let mut buf = BytesMut::with_capacity(32);
    let mut heartbeat = 5;
    // TODO do we want timeout if waiting for ';'?
    loop {
        info!("top of loop");
        // TODO: move read loop to its own fuction
        let msg = loop {
            socket.readable().await?;

            info!("top of read loop");
            let n = socket.read_buf(&mut buf).await?;
            info!("buf: {buf:?}");
            if n == 0 {
                info!("n == 0 heartbeat: {heartbeat}");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                if heartbeat < 0 {
                    return Ok("heartbeat timeout".to_string());
                } else {
                    heartbeat -= 1;
                }
                continue;
            } else if Some(&59) == buf.last() {
                let msg = str::from_utf8(&buf[..buf.len() - 1])?;
                info!("msg: {msg}");
                break msg
            } else {
                info!("else branch");
                continue;
            }
        };
        // TODO: move write section to its own fuction 
        let reply = Event::AgentCommand(port, msg.to_string());
        info!("agent reply: {reply:?}");
        event_sender.send(reply)?;
        info!("post event_sender send");
        // TODO 100 here is the AGENT_UPDATE_MILLIS, which should be configured at top level
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        info!("post sleep");
        let reply = retry_recv(rx, 0).await;
        socket.writable().await?;
        socket.write_all(reply.as_bytes()).await?;
        info!("post socket write_all");
        buf = BytesMut::with_capacity(32);
    }
}
