use clap::Args;
use std::error::Error as StdError;
use std::path::PathBuf;

use crate::libs::create_params::DEFAULT_DELAY_MS;
use crate::libs::global_config::GLOBAL_CONFIG_FILE_NAME;
use crate::libs::global_config::GlobalConfig;
use crate::libs::global_config::GlobalConfigBuilder;
use crate::libs::preflight_type::PreflightType;
use crate::middlewares::logger::VerbosityLevel;
use crate::server::params::DEFAULT_ADMIN_BASE_URL;
use crate::server::params::DEFAULT_CORS_ENABLED;
use crate::server::params::DEFAULT_HOST;
use crate::server::params::DEFAULT_HTTPS_PORT;
use crate::server::params::DEFAULT_MOCKS_DIR_NAME;
use crate::server::params::DEFAULT_PORT;

#[derive(Args, Debug, Clone)]
pub struct InitParams {
    #[arg(
        long,
        short,
        default_value_t = false,
        help = "Should the init process be interactive"
    )]
    interactive: bool,
}

impl InitParams {
    fn get_home_dir(&self) -> Result<PathBuf, anyhow::Error> {
        return std::env::home_dir().ok_or_else(|| anyhow::Error::msg("Failed to get home dir"));
    }

    fn get_user_config_path(&self) -> Result<PathBuf, anyhow::Error> {
        let home_dir = self.get_home_dir()?;
        Ok(home_dir.join(GLOBAL_CONFIG_FILE_NAME))
    }

    async fn ask_to_overwrite(&self) -> bool {
        let Ok(config_file_path) = self.get_user_config_path() else {
            return false;
        };
        let prompt = format!(
            "Are you sure you want to overwrite the file {}?",
            config_file_path.display()
        );
        tokio::task::spawn_blocking(move || {
            dialoguer::Confirm::new()
                .with_prompt(prompt)
                .interact()
                .unwrap_or(false)
        })
        .await
        .unwrap_or(false)
    }

    async fn check_if_config_exists(&self) -> Result<bool, anyhow::Error> {
        let Ok(metadata) = tokio::fs::metadata(self.get_user_config_path()?).await else {
            return Ok(false);
        };

        let exists = metadata.is_file() || metadata.is_dir() || metadata.is_symlink();

        Ok(exists)
    }

    async fn init_with_defaults(&self) -> Result<(), anyhow::Error> {
        let global_config = GlobalConfig::fair_defaults();

        let as_str = serde_json::to_string_pretty(&global_config)?;
        tokio::fs::write(self.get_user_config_path()?, as_str).await?;

        Ok(())
    }

    fn ask_host() -> String {
        dialoguer::Input::new()
            .with_initial_text(DEFAULT_HOST.to_string())
            .default(DEFAULT_HOST.to_string())
            .with_prompt("Specify a host/ip to work on")
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_HOST.to_string())
    }

    fn ask_port() -> u16 {
        let port_input = dialoguer::Input::new()
            .with_initial_text(DEFAULT_PORT.to_string())
            .default(DEFAULT_PORT.to_string())
            .with_prompt("Specity a port to listen on")
            .validate_with(|v: &String| -> Result<(), &'static str> {
                v.parse::<u16>()
                    .map(|_| ())
                    .map_err(|_| "Invalid port number")
            })
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_PORT.to_string());
        port_input.parse().unwrap_or(DEFAULT_PORT)
    }

    fn ask_https_port() -> u16 {
        let https_port_input = dialoguer::Input::new()
            .with_initial_text(DEFAULT_HTTPS_PORT.to_string())
            .default(DEFAULT_HTTPS_PORT.to_string())
            .with_prompt("Specify https port to listen on")
            .validate_with(|v: &String| -> Result<(), &'static str> {
                v.parse::<u16>()
                    .map(|_| ())
                    .map_err(|_| "Invalid port number")
            })
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_HTTPS_PORT.to_string());
        https_port_input.parse().unwrap_or(DEFAULT_HTTPS_PORT)
    }

    fn ask_mocks() -> String {
        dialoguer::Input::new()
            .with_initial_text(DEFAULT_MOCKS_DIR_NAME.to_string())
            .default(DEFAULT_MOCKS_DIR_NAME.to_string())
            .with_prompt("Specify a path to mocks, absolute or relative")
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_MOCKS_DIR_NAME.to_string())
    }

    fn ask_cors() -> bool {
        dialoguer::Confirm::new()
            .default(DEFAULT_CORS_ENABLED)
            .with_prompt("Enable CORS headers")
            .interact()
            .unwrap_or(false)
    }

    fn ask_preflight() -> PreflightType {
        let items_to_select = vec![PreflightType::Permissive, PreflightType::Mirror];
        let idx = dialoguer::Select::new()
            .with_prompt("Select preflight behavior")
            .default(0)
            .items(&items_to_select)
            .interact()
            .unwrap_or(0);
        items_to_select
            .get(idx)
            .cloned()
            .unwrap_or(PreflightType::Permissive)
    }

    fn ask_delay() -> u64 {
        let delay_input = dialoguer::Input::new()
            .with_initial_text(DEFAULT_DELAY_MS.to_string())
            .default(DEFAULT_DELAY_MS.to_string())
            .with_prompt("Specify delay in milliseconds before server starts responding")
            .validate_with(|v: &String| -> Result<(), &'static str> {
                v.parse::<u16>()
                    .map(|_| ())
                    .map_err(|_| "Choose value between 0 and 2^64 - 1")
            })
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_DELAY_MS.to_string());
        delay_input.parse().unwrap_or(DEFAULT_DELAY_MS)
    }

    fn ask_admin_base_url() -> String {
        dialoguer::Input::new()
            .with_initial_text(DEFAULT_ADMIN_BASE_URL.to_string())
            .default(DEFAULT_ADMIN_BASE_URL.to_string())
            .with_prompt("Specify default admin base URL")
            .interact_text()
            .unwrap_or_else(|_| DEFAULT_ADMIN_BASE_URL.to_string())
    }

    fn ask_log_request_level() -> VerbosityLevel {
        let items_to_select = vec![
            VerbosityLevel::Info,
            VerbosityLevel::Debug,
            VerbosityLevel::Trace,
        ];
        let idx = dialoguer::Select::new()
            .with_prompt("Select log request level")
            .default(0)
            .items(&items_to_select)
            .interact()
            .unwrap_or(0);
        items_to_select
            .get(idx)
            .cloned()
            .unwrap_or(VerbosityLevel::Info)
    }

    fn ask_verbosity_level() -> VerbosityLevel {
        let items_to_select = vec![
            VerbosityLevel::Info,
            VerbosityLevel::Debug,
            VerbosityLevel::Trace,
        ];
        let idx = dialoguer::Select::new()
            .with_prompt("Select verbosity level")
            .default(0)
            .items(&items_to_select)
            .interact()
            .unwrap_or(0);
        items_to_select
            .get(idx)
            .cloned()
            .unwrap_or(VerbosityLevel::Info)
    }

    async fn init_interactive(&self) -> Result<(), anyhow::Error> {
        let global_config = tokio::task::spawn_blocking(|| {
            let host = Self::ask_host();
            let port: u16 = Self::ask_port();
            let https_port: u16 = Self::ask_https_port();
            let mocks: String = Self::ask_mocks();
            let cors: bool = Self::ask_cors();
            let preflight: PreflightType = Self::ask_preflight();
            let delay_ms: u64 = Self::ask_delay();
            let admin_base_url: String = Self::ask_admin_base_url();
            let log_request: VerbosityLevel = Self::ask_log_request_level();
            let verbosity: VerbosityLevel = Self::ask_verbosity_level();

            GlobalConfig::default()
                .with_host(host)
                .with_port(port)
                .with_https_port(https_port)
                .with_mocks(mocks)
                .with_cors(cors)
                .with_preflight(preflight)
                .with_delay_ms(delay_ms)
                .with_admin_base_url(admin_base_url)
                .with_log_request(log_request)
                .with_verbosity(verbosity)
        })
        .await
        .unwrap_or_else(|_| GlobalConfig::fair_defaults());

        let as_str = serde_json::to_string_pretty(&global_config)?;
        tokio::fs::write(self.get_user_config_path()?, as_str).await?;

        Ok(())
    }

    pub async fn init(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
        let exists = self.check_if_config_exists().await?;

        let proceed = match exists {
            true => self.ask_to_overwrite().await,
            false => true,
        };

        if !proceed {
            return Ok(());
        }

        let result = match self.interactive {
            true => self.init_interactive().await,
            false => self.init_with_defaults().await,
        };

        println!(
            "Config saved to {}",
            self.get_user_config_path()?.to_string_lossy().to_string()
        );

        result.map_err(Box::from)
    }
}

trait FairDefaults<T> {
    fn fair_defaults() -> T;
}

impl FairDefaults<GlobalConfig> for GlobalConfig {
    fn fair_defaults() -> GlobalConfig {
        GlobalConfig::default()
            .with_host(DEFAULT_HOST.to_string())
            .with_port(DEFAULT_PORT)
            .with_https_port(DEFAULT_HTTPS_PORT)
            .with_mocks(DEFAULT_MOCKS_DIR_NAME.to_string())
            .with_cors(DEFAULT_CORS_ENABLED)
            .with_preflight(PreflightType::Permissive)
            .with_delay_ms(DEFAULT_DELAY_MS)
            .with_admin_base_url(DEFAULT_ADMIN_BASE_URL.to_string())
            .with_log_request(VerbosityLevel::Info)
            .with_verbosity(VerbosityLevel::Info)
    }
}
