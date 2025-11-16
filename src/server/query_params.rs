use std::{collections::HashMap, str::FromStr};

pub struct QueryParams {
    map: std::collections::HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.map.get(key)
    }
}

impl FromStr for QueryParams {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut source_string = s.to_string();
        if source_string.starts_with("?") {
            source_string = s[1..].to_string();
        }

        let pairs_str: Vec<String> = source_string.split('&').map(|x| x.to_string()).collect();
        let mut result = HashMap::<String, Vec<String>>::new();

        for pair in pairs_str {
            let pair_vec: Vec<String> = pair.split('=').map(|x| x.to_string()).collect();
            let key = pair_vec.get(0);
            let value = pair_vec.get(1);
            if let (Some(key), Some(value)) = (key, value) {
                result.entry(key.clone()).or_default().push(value.clone());
            } else if let Some(key) = key {
                result.entry(key.clone()).or_default().push("".to_string());
            }
        }

        return Ok(QueryParams { map: result });
    }
}
