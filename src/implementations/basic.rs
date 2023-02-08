use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use std::path::Path;
use tokio::fs;
use tokio::io;
use crate::implementations::ifc::Database;

pub struct Basic {
    file: Option<fs::File>,
}

impl Basic {
    pub fn new() -> Basic {
        Basic {
            file: None,
        }
    }
}

#[async_trait]
impl Database for Basic {
    async fn init(&mut self) -> io::Result<()> {
        let path = Path::new("./.basic");
        fs::create_dir_all(path).await?;

        let file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path.join("data"))
            .await?;

        self.file = Some(file);

        Ok(())
    }

    async fn reset(&mut self) -> io::Result<()> {
        let file = self.file.as_mut().expect("Uninitialized database");
        file.set_len(0).await?;

        Ok(())
    }

    async fn write(&mut self, key: &str, value: &str) -> io::Result<()> {
        let file = self.file.as_mut().expect("Uninitialized database");

        file.write(format!("{}\u{038D}{}\n", key, value).as_bytes()).await?;

        Ok(())
    }

    async fn read(&mut self, key: &str) -> io::Result<Option<String>> {
        let file = self.file.as_mut().expect("Uninitialized database");
        file.rewind().await?;
        let mut reader = io::BufReader::new(file);

        let mut found: Option<String> = None;

        loop {
            let mut buffer = String::new();
            let bytes = reader.read_line(&mut buffer).await?;

            // Reached EOF
            if bytes == 0 {
                break;
            }

            let curr_key: String = buffer.chars().take_while(|c| c != &'\u{038D}').collect();

            if curr_key == key {
                let value: String = buffer.chars().skip(curr_key.len() + 1).collect();

                found = Some(value);
            }
        }

        Ok(found)
    }
}
