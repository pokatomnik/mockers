use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

pub(crate) struct YamlBuilder {
    static_yaml: String,
}

impl YamlBuilder {
    pub fn add_new_line(&mut self) {
        self.static_yaml.push('\n');
    }

    pub fn add_servers<S, I>(&mut self, servers: S)
    where
        I: Into<String>,
        S: IntoIterator<Item = I>,
    {
        self.static_yaml.push_str("servers:");
        self.add_new_line();
        for server in servers {
            let server_entry = format!("  - url: \"{}\"", server.into());
            self.static_yaml.push_str(server_entry.as_str());
            self.add_new_line()
        }
    }
}

pub(crate) type YamlBuilderCreate = Result<YamlBuilder, Utf8Error>;

impl TryFrom<&[u8]> for YamlBuilder {
    type Error = Utf8Error;

    fn try_from(value: &[u8]) -> YamlBuilderCreate {
        str::from_utf8(value).map(|static_yaml| Self {
            static_yaml: static_yaml.into(),
        })
    }
}

impl Display for YamlBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.static_yaml)
    }
}
