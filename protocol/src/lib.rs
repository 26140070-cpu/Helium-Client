mod command;
mod logging;

pub use command::{Command, ProtocolError};
pub use logging::{init_file_logger, LoggerError};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const SOCKET_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7878);
