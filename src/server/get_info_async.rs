use crate::libs::absolute_mocks_path::AbsoluteMocksPath;
use crate::libs::get_info_async::GetInfoAsync;
use crate::server::params::ServerParams;

impl GetInfoAsync for ServerParams {
    async fn get_help(&self, title: &str) -> String {
        let mut buf = String::from(format!("{}:{}", title, Self::EOL));
        buf.push_str(&format!("================={}", Self::EOL));
        let host_info = format!(
            "Host:{}{}{}",
            Self::TAB.repeat(3),
            self.get_host().await,
            Self::EOL
        );
        let port_info = format!(
            "Port:{}{}{}",
            Self::TAB.repeat(3),
            self.get_port().await,
            Self::EOL
        );
        let mocks_path_info = format!(
            "Mocks path:{}{}{}",
            Self::TAB.repeat(2),
            self.get_absolute_mocks_path()
                .await
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let cors_info = format!(
            "Cors:{}{}{}",
            Self::TAB.repeat(3),
            self.cors().await,
            Self::EOL
        );
        let preflight_info = format!(
            "Preflight:{}{}{}",
            Self::TAB.repeat(2),
            self.preflight()
                .await
                .map(|pt| pt.to_string())
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL
        );
        let delay_ms_info = format!(
            "Delay in milliseconds:{}{}{}",
            Self::TAB,
            self.delay_ms().await,
            Self::EOL
        );
        let origin_info = format!(
            "Origin:{}{}{}",
            Self::TAB.repeat(3),
            self.origin()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let admin_base_url_info = format!(
            "Admin base URL:{}{}{}",
            Self::TAB.repeat(2),
            self.admin_base_url()
                .await
                .unwrap_or_else(|| Self::UNSET.to_string()),
            Self::EOL,
        );
        let log_request_info = format!(
            "Requests log level:{}{}{}",
            Self::TAB,
            self.log_request().await,
            Self::EOL,
        );
        let verbosity_level_info = format!(
            "Verbosity level:{}{}{}",
            Self::TAB,
            self.verbosity_level().await,
            Self::EOL
        );

        buf.push_str(&host_info);
        buf.push_str(&port_info);
        buf.push_str(&mocks_path_info);
        buf.push_str(&cors_info);
        buf.push_str(&preflight_info);
        buf.push_str(&delay_ms_info);
        buf.push_str(&origin_info);
        buf.push_str(&admin_base_url_info);
        buf.push_str(&log_request_info);
        buf.push_str(&verbosity_level_info);

        buf
    }
}
