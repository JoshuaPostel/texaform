pub mod dog;
pub mod fabricator;
pub mod hud;
pub mod laser_cutter;
pub mod smelter;

use crate::AppResult;
use crate::entities::Entity;
use crate::entities::PickResult;
use crate::event::Event;
use crate::surface::Power;
use crate::surface::Surface;
use crate::surface::grid::Grid;
use crate::surface::state::GameState;
use crate::tcp::handle_socket;
use crate::widgets::text_box::TextBox;

use ratatui::Frame;
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::widgets::WidgetRef;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::AbortHandle;

use std::fmt::Display;
use std::net::SocketAddr;

use ratatui::widgets::List;
use serde_with::serde_as;

#[typetag::serde]
pub trait Agent: std::fmt::Debug + WidgetRef {
    fn handle_message(
        &mut self,
        pos: &Position,
        grid: &mut Grid,
        game_state: &mut GameState,
        msg: String,
    ) -> UpdateEnum;

    fn entity(&self) -> Entity;
    fn tick(&mut self, _power: &mut Power) {}
    fn on_init(&self, _power: &mut Power) {}
    fn render_surface_cell(&self, offset: &Position, cell: &mut Cell) {
        cell.bg = Color::DarkGray;
        if offset == &Position::new(0, 0) {
            cell.set_char(self.entity().character());
        } else {
            cell.set_char(self.entity().character().to_ascii_lowercase());
        }
    }
    // this is good for rendering effects that follow a moving entity.
    // do we need a method for rendering effects in a static location?
    fn render_fx(
        &mut self,
        _grid_position: &Position,
        _frame: &mut Frame,
        _area: Rect,
        _prev_tick: core::time::Duration,
    ) {
    }
    /// implement if the entity is pickable
    fn pick(&mut self, _c: char) -> PickResult {
        PickResult::noop()
    }

    /// implement if dog can place entites into this entity
    fn placable(&self, _entity: &Entity) -> bool {
        false
    }
    fn place(&mut self, _entity: Entity) {}

    // TODO replace with requiring Default trait
    fn new() -> Self
    where
        Self: Sized;

    fn integrity(&self) -> usize {
        100
    }
}

pub enum UpdateEnum {
    BuildAgent {
        pos: Position,
        agent: Box<dyn Agent + 'static>,
    },
    BuildEntity {
        pos: Position,
        entity: Entity,
    },
    Reply(String),
    Move(Position),
    Research,
}

impl UpdateEnum {
    pub fn reply(reply: impl std::fmt::Display) -> UpdateEnum {
        UpdateEnum::Reply(reply.to_string())
    }

    pub fn okay() -> UpdateEnum {
        UpdateEnum::Reply("OKAY".to_string())
    }
}

#[derive(Debug)]
pub struct Update {
    pub reply: String,
    // TODO is this used?
    pub position: Option<Position>,
    //pub build_agent: Option<Box<dyn Agent + 'static>>,
    pub build_agent: Option<Box<dyn Agent + 'static>>,
    pub build_intermediate: Option<Entity>,
}

impl Update {
    pub fn new(reply: impl std::fmt::Display, position: Position) -> Update {
        Update {
            reply: reply.to_string(),
            position: Some(position),
            build_agent: None,
            build_intermediate: None,
        }
    }
    pub fn position(position: Position) -> Update {
        Update {
            reply: "OKAY".to_string(),
            position: Some(position),
            build_agent: None,
            build_intermediate: None,
        }
    }
    pub fn reply(reply: impl std::fmt::Display) -> Update {
        Update {
            reply: reply.to_string(),
            position: None,
            build_agent: None,
            build_intermediate: None,
        }
    }
    pub fn noop() -> Update {
        Update {
            reply: "OKAY".to_string(),
            position: None,
            build_agent: None,
            build_intermediate: None,
        }
    }
    pub fn build_agent(position: Position, agent: impl Agent + 'static) -> Update {
        Update {
            reply: "NEED TO RETURN PORT".to_string(),
            position: Some(position),
            build_agent: Some(Box::new(agent)),
            build_intermediate: None,
        }
    }
    pub fn build_intermediate(position: Position, entity: Entity) -> Update {
        Update {
            reply: "NEED TO RETURN PORT".to_string(),
            position: Some(position),
            build_agent: None,
            build_intermediate: Some(entity),
        }
    }
}

#[derive(Debug)]
struct DropHandle {
    handle: AbortHandle,
}

impl Drop for DropHandle {
    fn drop(&mut self) {
        self.handle.abort()
    }
}

// write only ring buffer
#[derive(Debug)]
pub struct CommLogs {
    data: [(String, String); 256],
    write_idx: u8,
}

impl Default for CommLogs {
    fn default() -> CommLogs {
        CommLogs {
            data: [(); 256].map(|_| ("".to_string(), "".to_string())),
            write_idx: 0,
        }
    }
}

impl CommLogs {
    pub fn push(&mut self, item: (String, String)) {
        self.data[self.write_idx as usize] = item;
        self.write_idx = self.write_idx.wrapping_add(1);
    }

    pub fn previous_command(&self, n_previous: u8) -> String {
        let idx = self.write_idx.wrapping_sub(n_previous) as usize;
        let (command, _) = &self.data[idx];
        command.clone()
    }

    pub fn list(&self, n: u8, width: usize) -> List<'_> {
        let width = (width / 2).saturating_sub(2);
        (1..n + 1)
            .map(|i| {
                let idx = self.write_idx.wrapping_sub(i) as usize;
                let (command, response) = &self.data[idx];
                format!(
                    "{:width$}|{}",
                    command.chars().take(width).collect::<String>(),
                    response,
                    width = width
                )
            })
            .collect::<List>()
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Comms {
    // TODO state to be saved/loaded
    pub port: usize,
    pub entity: Entity,
    pub position: Option<Position>,
    pub location: Option<Rect>,
    // TODO limit the size
    //pub log: Vec<(String, String)>,
    #[serde(skip)]
    pub log: CommLogs,

    pub address: Option<SocketAddr>,
    pub text_box: TextBox,

    #[serde(skip)]
    pub reply_sender: Option<Sender<String>>,
    #[serde(skip)]
    pub(self) drop_handle: Option<DropHandle>,
}

impl Comms {
    pub async fn init(&mut self, event_sender: &UnboundedSender<Event>) {
        if self.reply_sender.is_none() {
            let event_sender = event_sender.clone();
            let (reply_sender, drop_handle): (Sender<String>, DropHandle) =
                spawn_handler(self.port, event_sender)
                    .await
                    .unwrap_or_else(|_| panic!("TODO {}", self.port));
            self.reply_sender = Some(reply_sender);
            self.drop_handle = Some(drop_handle);
        }
    }

    // fn sender(&mut self, surface: &Surface) -> &Sender<String> {
    pub fn sender(&mut self) -> &Sender<String> {
        // if self.reply_sender.is_none() {
        //     let event_sender = surface.event_sender.clone();
        //     let sender: Sender<String> = spawn_handler(self.port, event_sender).await.expect("TODO");
        //     self.reply_sender = Some(sender);
        // }
        self.reply_sender
            .as_ref()
            .expect("reply_sender inited properly")
    }

    pub async fn new(surface: &Surface, location: Option<Rect>, agent_kind: Entity) -> Comms {
        let port = surface.next_available_port();
        tracing::info!("here: {port}");
        let event_sender = surface.event_sender.clone();
        let (reply_sender, drop_handle): (Sender<String>, DropHandle) =
            spawn_handler(port, event_sender).await.expect("TODO");
        let (position, location) = match location {
            Some(location) => (Some(Position::new(location.x, location.y)), Some(location)),
            None => (None, None),
        };
        Comms {
            address: None,
            entity: agent_kind,
            position,
            location,
            port,
            log: CommLogs::default(),
            text_box: TextBox::new().clear_on_enter(true),
            reply_sender: Some(reply_sender),
            drop_handle: Some(drop_handle),
        }
    }
}

async fn spawn_handler<R: Display + Send + 'static>(
    port: usize,
    event_sender: UnboundedSender<Event>,
) -> AppResult<(Sender<R>, DropHandle)> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    tracing::info!("started tcp listener on port {port}");

    let (tx, mut rx) = mpsc::channel(32);

    // handling one connection at a time for now
    let join_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let addr = socket.peer_addr().expect("should have peer address");
            tracing::info!("peer address: {}", addr);
            event_sender
                .send(Event::AgentConnection(port, addr))
                .expect("TODO");
            tracing::info!("here asdf");
            match handle_socket(port, socket, event_sender.clone(), &mut rx).await {
                Err(e) => {
                    tracing::warn!("error hadling socket: {e}");
                    event_sender
                        .send(Event::AgentDisconect(port))
                        .expect("TODO");
                    tracing::warn!("here");
                }
                Ok(_) => {
                    event_sender
                        .send(Event::AgentDisconect(port))
                        .expect("TODO");
                }
            }
        }
    });

    let drop_handle = DropHandle {
        handle: join_handle.abort_handle(),
    };
    Ok((tx, drop_handle))
}

pub const DOCUMENTATION: &str = r#"AGENTS

  agents are controled by sending short text commands such as `MOVE` and `TURN L`

  agents will reply to every command with a short text response such as `OKAY`, `AREA I..`, and `ERRR location occupied`

  for a list of commands and responses see the agent's documentation


  MANUAL COMMUNICATION 

    1. click on an agent to select it
    2. click on the `Command Line` in the bottom right
    3. type in the desired command and press [ENTER]

    Note: the `Command Line` will be disabled if the agent is connteted via TCP (see below)


  AUTOMATED COMMUNICATION

    each agent has a unique port and can receive commands via Transmission Control Protocol (TCP)

    commands sent over TCP must:
      1. end in a semicolon `;`
      2. be 32 bytes or less
      3. all characters confrom to the American Standard Code for Information Interchange (ASCII)

    reponses received over TCP will follow the same rules


    example python implemenataion:

        ```
        import socket

        def send(socket, msg: str) -> str:
            msg_bytes = str.encode(msg + ";")
            socket.sendall(msg_bytes)
            buffer = b""
            while b";" not in buffer:
                buffer += socket.recv(32)
            response = buffer.decode("utf-8")
            return response[:-1]

        dog = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        dog.connect(("localhost", 3335))

        response = send(dog, "MOVE")
        print(response) # OKAY
        ```
    
"#;
