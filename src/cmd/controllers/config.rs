use crate::entities::global_config::GlobalConfigAPI;
use crate::libs::get_info_async::GetInfoAsync;
use clap::Args;
use tokio::sync::OnceCell;

#[derive(Args, Debug, Clone)]
pub(crate) struct ConfigParams {
    #[clap(skip)]
    global_config: OnceCell<GlobalConfigAPI>,
}

impl ConfigParams {
    pub async fn show_config(&self) {
        let help_str = self.get_help("Global config").await;
        println!("{}", help_str);
    }
}

impl GetInfoAsync for ConfigParams {
    async fn get_help(&self, title: &str) -> String {
        self.global_config
            .get_or_init(GlobalConfigAPI::new)
            .await
            .get_help(title)
            .await
    }
}
