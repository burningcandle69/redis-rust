use crate::Error;
use crate::frame::Frame;
use std::collections::VecDeque;

mod errors;
mod list;
mod persistence;
mod replication;
pub mod server;
mod stream;
mod string;
mod transaction;
mod pubsub;

type Result = std::result::Result<Frame, Error>;

pub type Args = VecDeque<String>;
