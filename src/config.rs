use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderConfig {
	pub entry_name: String,
	pub kernel_path: String,
	pub kernel_arguments: String,
	pub protocol: Option<String>,
	pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub images: Vec<String>,
	pub bootloader: BootloaderConfig,
}

pub fn create_limine_config(config: &BootloaderConfig) -> String {
	format!(
		r#"TIMEOUT={}
SERIAL=yes
VERBOSE=yes
:{}
PROTOCOL={}
KERNEL_CMDLINE={}
KERNEL_PATH=boot://{}"#,
		config.timeout.unwrap_or_default(),
		config.entry_name,
		config.protocol.as_ref().unwrap_or(&"linux".to_string()),
		config.kernel_arguments,
		config.kernel_path
	)
}
