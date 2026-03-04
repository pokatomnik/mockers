pub(crate) trait GetInfoAsync {
    const UNSET: &'static str = "[unset]";
    const TAB: &str = "\t";
    const EOL: &'static str = "\n";

    async fn get_help(&self, title: &str) -> String;
}
