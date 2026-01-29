use std::{fs::Metadata, path::PathBuf, vec::IntoIter};

pub(crate) struct FSWalker {
    stack: Vec<PathBuf>,
}

impl FSWalker {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FSWalker {
            stack: vec![root.into()],
        }
    }

    async fn read_dir(from: &PathBuf) -> Vec<PathBuf> {
        let mut res = Vec::new();
        if let Ok(mut rd) = tokio::fs::read_dir(from).await {
            while let Ok(Some(entry)) = rd.next_entry().await {
                res.push(entry.path());
            }
        }
        res
    }

    pub async fn next(&mut self) -> Option<(Metadata, PathBuf)> {
        while let Some(current) = self.stack.pop() {
            if let Ok(md) = tokio::fs::metadata(&current).await {
                if md.is_dir() {
                    let contents = Self::read_dir(&current).await;
                    self.stack.extend(contents);
                }

                return Some((md, current));
            }
        }

        None
    }

    pub async fn into_iter(mut self) -> IntoIter<(Metadata, PathBuf)> {
        let mut result = Vec::new();
        while let Some(tuple) = self.next().await {
            result.push(tuple);
        }
        result.into_iter()
    }
}
