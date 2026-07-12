use crate::libs::get_info_async::GetInfoAsync;
use crate::libs::get_info_table::render_help_table;
use crate::server::params::ServerParams;
use crate::use_cases::absolute_mocks_path::AbsoluteMocksPath;

impl GetInfoAsync for ServerParams {
    async fn get_help(&self, title: &str) -> String {
        render_help_table(
            title,
            vec![
                ("Host", self.get_host().await.to_string()),
                ("Port", self.get_port().await.to_string()),
                (
                    "Mocks path",
                    self.get_absolute_mocks_path()
                        .await
                        .map(|v| v.to_string_lossy().to_string())
                        .unwrap_or_else(|| Self::UNSET.to_string()),
                ),
                ("Cors", self.cors().await.to_string()),
                (
                    "Preflight",
                    self.preflight()
                        .await
                        .map(|pt| pt.to_string())
                        .unwrap_or_else(|| Self::UNSET.to_string()),
                ),
                ("Delay in milliseconds", self.delay_ms().await.to_string()),
                (
                    "Origin",
                    self.origin()
                        .await
                        .unwrap_or_else(|| Self::UNSET.to_string()),
                ),
                (
                    "Admin base URL",
                    self.admin_base_url()
                        .await
                        .unwrap_or_else(|| Self::UNSET.to_string()),
                ),
                ("Requests log level", self.log_request().await.to_string()),
                ("Verbosity level", self.verbosity_level().await.to_string()),
            ],
        )
    }
}
