use crate::reminder::{ChannelData, Reminder};
use ahash::AHasher;
use chrono_tz::{ParseError, Tz};
use serenity::model::id::ChannelId;
use slotmap::DefaultKey;
use std::{collections::HashMap, hash::BuildHasherDefault, io::SeekFrom, iter::repeat, pin::Pin};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeek, AsyncWriteExt},
};

pub struct Db {
    file: File,
    data: HashMap<ChannelId, ChannelData, BuildHasherDefault<AHasher>>,
}

impl Db {
    pub async fn open(db_path: &str) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .append(false)
            .open(&db_path)
            .await
            .expect("Error opening database file to read");

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .expect("Error reading database file");

        let data = serde_json::from_str(&contents).unwrap_or_else(|_| HashMap::default());

        Self { file, data }
    }

    async fn persist(&mut self) {
        Pin::new(&mut self.file)
            .start_seek(SeekFrom::Start(0))
            .expect("Error seeking to start of database file");
        self.file
            .write_all(
                serde_json::to_string(&self.data)
                    .expect("Error serializing data")
                    .as_bytes(),
            )
            .await
            .expect("Error writing to database file");
    }

    pub async fn insert(&mut self, key: ChannelId, data: Reminder) -> DefaultKey {
        let key = self
            .data
            .entry(key)
            .or_insert_with(ChannelData::default)
            .reminders
            .insert(data);
        self.persist().await;

        key
    }

    pub async fn remove(&mut self, key: ChannelId, inner_key: DefaultKey) {
        self.data.entry(key).and_modify(|r| {
            r.reminders.remove(inner_key);
        });
        self.persist().await;
    }

    pub fn tz(&self, key: ChannelId) -> Option<Tz> {
        self.data.get(&key).map(|cd| cd.tz)
    }

    pub async fn set_tz(&mut self, key: ChannelId, tz_str: &str) -> Result<(), ParseError> {
        self.data
            .entry(key)
            .or_insert_with(ChannelData::default)
            .set_tz(tz_str)?;
        self.persist().await;

        Ok(())
    }

    pub fn has_reminder(&self, key: ChannelId, inner_key: DefaultKey) -> bool {
        self.data
            .get(&key)
            .map_or(false, |cd| cd.reminders.contains_key(inner_key))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChannelId, Tz, DefaultKey, &Reminder)> {
        self.data.iter().flat_map(|(k, cd)| {
            repeat((k, cd.tz))
                .zip(cd.reminders.iter())
                .map(|((k, t), (d, v))| (k, t, d, v))
        })
    }

    pub fn channel_iter(
        &self,
        key: ChannelId,
    ) -> Option<impl Iterator<Item = (DefaultKey, &Reminder)>> {
        self.data.get(&key).map(|cd| cd.reminders.iter())
    }
}
