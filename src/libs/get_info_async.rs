pub(crate) trait GetInfoAsync {
    const UNSET: &'static str = "[unset]";

    async fn get_help(&self, title: &str) -> String;
}
