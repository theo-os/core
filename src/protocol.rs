use serde::{Deserialize, Serialize};
use tokio::process::Child;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
	Shutdown,
	Start(String),
	Stop(String),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum State {
	Stopping,
	Stopped,
	Starting,
	Running,
	Failed,
	Restarting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
	pub command: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Service {
	pub config: ServiceConfig,
	pub current_state: State,
	#[serde(skip_serializing)]
	pub child: Option<Child>,
}
