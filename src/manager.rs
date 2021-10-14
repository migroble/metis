use crate::{
    db::Db,
    reminder::{ChannelData, Reminder},
};
use chrono::Utc;
use chrono_tz::{Etc::UTC, ParseError, Tz};
use serenity::{model::id::ChannelId, prelude::*};
use slotmap::DefaultKey;
use std::sync::Arc;
use tokio::{sync::RwLock, time::sleep};

pub struct Manager {
    db: Arc<RwLock<Db>>,
}

impl Manager {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            db: Arc::new(RwLock::new(Db::open(db_path).await)),
        }
    }

    pub async fn set_channel_tz(
        &self,
        channel_id: ChannelId,
        tz_str: &str,
    ) -> Result<(), ParseError> {
        self.db.write().await.set_tz(channel_id, tz_str).await?;

        Ok(())
    }

    pub async fn channel_tz(&self, channel_id: ChannelId) -> Option<Tz> {
        self.db.read().await.tz(channel_id)
    }

    pub async fn channel_data(&self, channel_id: ChannelId) -> Option<ChannelData> {
        self.db
            .read()
            .await
            .channel_data(channel_id)
            .map(|cd| cd.clone())
    }

    pub fn start_reminding(
        &self,
        ctx: Arc<Context>,
        channel_id: ChannelId,
        tz: Tz,
        key: DefaultKey,
        reminder: Reminder,
    ) {
        let db = Arc::clone(&self.db);
        tokio::spawn(async move {
            let sched = reminder.sched;

            for datetime in sched.upcoming(tz) {
                // We ensure chrono::Duration::to_std cannot panic by checking that the number
                // of seconds is positive. Additionally, a non-positive number of seconds means
                // we don't have to sleep
                let remaining = datetime.signed_duration_since(Utc::now());
                if remaining.num_seconds() > 0 {
                    sleep(remaining.to_std().unwrap()).await;
                }

                if db.read().await.has_reminder(channel_id, key) {
                    // TODO: Log when no permission to send message rather than panic
                    channel_id
                        .send_message(&ctx, |m| m.content(&reminder.msg))
                        .await
                        .expect("Error sending reminder");
                } else {
                    // Another task removed this reminder so we stop
                    break;
                }
            }

            // If there are no more reminders, the entry is removed
            db.write().await.remove(channel_id, key).await;
        });
    }

    pub async fn start_reminders(&self, ctx: Arc<Context>) {
        self.db
            .read()
            .await
            .iter()
            .for_each(|(c, t, k, r)| self.start_reminding(Arc::clone(&ctx), *c, t, k, r.clone()));
    }

    pub async fn add_reminder(&self, ctx: Arc<Context>, channel_id: ChannelId, reminder: Reminder) {
        let key = self
            .db
            .write()
            .await
            .insert(channel_id, reminder.clone())
            .await;
        self.start_reminding(ctx, channel_id, UTC, key, reminder);
    }

    pub async fn remove_reminder(&self, channel_id: ChannelId, key: DefaultKey) {
        self.db.write().await.remove(channel_id, key).await;
    }
}
