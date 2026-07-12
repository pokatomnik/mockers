use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Tabled)]
pub(crate) struct HelpRow {
    #[tabled(rename = "Config entry")]
    field: String,

    #[tabled(rename = "Config value")]
    value: String,
}

pub(crate) fn render_help_table(title: &str, rows: Vec<(&str, String)>) -> String {
    let rows = rows
        .into_iter()
        .map(|(field, value)| HelpRow {
            field: field.to_string(),
            value,
        })
        .collect::<Vec<_>>();

    let table = Table::new(rows).with(Style::rounded()).to_string();

    format!("{}:\n{}\n", title, table)
}
