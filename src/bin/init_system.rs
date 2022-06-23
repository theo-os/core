#![feature(async_closure)]
use anyhow::{Context, Result};
use async_trait::async_trait;
use hyper::{Body, Request, Response, Server};
use routerify::{ext::RequestExt, Router, RouterService};
use std::collections::BTreeMap;
use std::sync::Arc;
use theos_kit::protocol::{Message, Service, ServiceConfig, State};
use tokio::{
	process::Command,
	sync::{mpsc, Mutex},
};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();
	let config = tokio::fs::read_to_string("services.json").await?;
	let services = Arc::new(Mutex::new(BTreeMap::<String, Service>::new()));

	let config: BTreeMap<String, ServiceConfig> =
		serde_json::from_str(&config)?;

	for service_config in config.clone() {
		let mut service = Service {
			config: service_config.1,
			current_state: State::Stopped,
			child: None,
		};

		service.start().await?;

		services.lock().await.insert(service_config.0, service);
	}

	info!("Initialized the daemon!");

	let (tx, rx) = mpsc::channel::<Message>(100);
	let rx = Arc::new(Mutex::new(rx));
	let tx = Arc::new(Mutex::new(tx));

	tokio::spawn(async move {
		loop {
			if let Ok(message) = rx.lock().await.try_recv() {
				match message {
					Message::Shutdown => {
						info!("Got: Shutdown signal");
						break;
					}
					Message::Start(unit_name) => {
						info!("Got start {:?}", unit_name);
						if let Some(service) =
							services.lock().await.get_mut(&unit_name)
						{
							service.start().await.unwrap();
						} else {
							error!(
								"Did not find a service named {}",
								unit_name
							);
						}
					}
					Message::Stop(unit_name) => {
						info!("Got stop {:?}", unit_name);
						if let Some(service) =
							services.lock().await.get_mut(&unit_name)
						{
							service.stop().await.unwrap();
						// This is our server object.
						} else {
							error!(
								"Did not find a service named {}",
								unit_name
							);
						}
					}
				}
			}
		}
	});

	tokio::spawn(async move {
		let addr = format!("127.0.0.1:{}", 6969);
		let tx = tx.clone();

		let router = Router::<Body, anyhow::Error>::builder()
			.data(tx.clone())
			.post("/shutdown", shutdown_handler)
			.build()
			.unwrap();
		let service = RouterService::new(router).unwrap();

		info!("API Server running on {}", addr);

		let server =
			Server::bind(&addr.parse().expect("Unable to parse host port"))
				.serve(service);

		if let Err(e) = server.await {
			error!("server error: {}", e);
		}
	});

	// Spawn boot service
	let boot_service = config
		.get("boot")
		.context("Failed to find 'boot' service.")
		.unwrap();

	loop {
		if let Err(_err) = Command::new(&boot_service.command[0])
			.args(
				boot_service
					.command
					.iter()
					.skip(1)
					.collect::<Vec<&String>>()
					.as_slice(),
			)
			.status()
			.await
		{}
	}
}

#[async_trait]
pub trait ActiveService {
	async fn start(&mut self) -> Result<()>;
	async fn stop(&mut self) -> Result<()>;
}

#[async_trait]
impl ActiveService for Service {
	async fn start(&mut self) -> Result<()> {
		self.current_state = State::Starting;

		let mut cmd = Command::new(&self.config.command[0]);
		cmd.args(
			self.config
				.command
				.iter()
				.skip(1)
				.collect::<Vec<&String>>()
				.as_slice(),
		);

		self.child = Some(cmd.spawn().context(format!(
			"Failed to spawn process for {:?}",
			&self.config.command[0],
		))?);
		self.current_state = State::Running;

		Ok(())
	}

	async fn stop(&mut self) -> Result<()> {
		self.current_state = State::Stopping;

		if let Some(child) = self.child.as_mut() {
			child.kill().await?;
		}

		self.current_state = State::Stopped;

		Ok(())
	}
}

async fn shutdown_handler(req: Request<Body>) -> Result<Response<Body>> {
	let mut state = req.data::<Arc<Mutex<mpsc::Sender<Message>>>>();
	let state = state.as_mut().unwrap();

	state.lock().await.send(Message::Shutdown).await?;

	Ok(Response::new(Body::from("Shutting down init")))
}
