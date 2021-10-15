use crate::{
    manager::Manager,
    reminder::{ChannelData, Reminder, ReminderType},
};
use chrono::TimeZone;
use chrono_tz::Tz;
use serenity::{
    builder::CreateInteractionResponseData,
    model::{
        id::ChannelId,
        interactions::{
            message_component::{ButtonStyle, ComponentType, MessageComponentInteraction},
            InteractionResponseType,
        },
    },
    prelude::*,
};
use slotmap::DefaultKey;
use std::{collections::HashMap, sync::Arc};

pub struct ReminderMenu {
    tz: Tz,
    reminders: HashMap<DefaultKey, Reminder>,
    selected: Option<String>,
}

impl ReminderMenu {
    pub async fn new(manager: &Manager, channel_id: ChannelId) -> Self {
        let channel = manager
            .channel_data(channel_id)
            .await
            .unwrap_or_else(ChannelData::default);
        let tz = channel.tz;
        let reminders = channel.reminders.into_iter().collect::<HashMap<_, _>>();

        Self {
            tz,
            reminders,
            selected: None,
        }
    }

    pub fn create<'a>(
        &self,
        message: &'a mut CreateInteractionResponseData,
    ) -> &'a mut CreateInteractionResponseData {
        if self.reminders.len() > 0 {
            message
                .content(format!("Channel timezone: {}", self.tz))
                .components(|comps| {
                    comps
                        .create_action_row(|ar| {
                            ar.create_select_menu(|sm| {
                                sm.min_values(0)
                                    .custom_id("menu-reminders")
                                    .options(|opts| {
                                        self.reminders.iter().fold(opts, |opts, (k, r)| {
                                            opts.create_option(|opt| {
                                                // This should never panic, any key should be
                                                // stringifiable
                                                let key = serde_json::to_string(k)
                                                    .expect("Error serializing key");
                                                let (info, datetime) = match &r.reminder_type {
                                                    ReminderType::Scheduled(sched) => (
                                                        "Repeating",
                                                        sched.upcoming(self.tz).next(),
                                                    ),
                                                    ReminderType::Once(datetime) => (
                                                        "One-shot",
                                                        Some(self.tz.from_utc_datetime(&datetime)),
                                                    ),
                                                };
                                                let datetime = datetime
                                                    .map(|t| t.to_rfc2822())
                                                    .unwrap_or("".to_string());

                                                opt.label(&r.msg)
                                                    .description(format!("{} ({})", datetime, info))
                                                    .value(key.clone())
                                                    .default_selection(
                                                        self.selected
                                                            .as_ref()
                                                            .map(|s| *s == key)
                                                            .unwrap_or(false),
                                                    )
                                            })
                                        })
                                    })
                            })
                        })
                        .create_action_row(|ar| {
                            ar.create_button(|b| {
                                let b = b.style(ButtonStyle::Danger).label("Delete");
                                if let Some(sel) = &self.selected {
                                    b.custom_id(format!("menu-{}", &sel))
                                } else {
                                    b.custom_id("menu-delete").disabled(true)
                                }
                            })
                        })
                })
        } else {
            message.content("no reminders").components(|comps| comps)
        }
    }

    pub async fn handle(
        &mut self,
        ctx: Arc<Context>,
        manager: &Manager,
        message: &MessageComponentInteraction,
    ) {
        match message.data.component_type {
            ComponentType::SelectMenu => {
                self.selected = message.data.values.get(0).cloned();
            }
            ComponentType::Button => {
                // This should never panic because the custom_id is always the valid json string
                // we generated in ReminderMenu::create
                let key =
                    serde_json::from_str(&message.data.custom_id.strip_prefix("menu-").unwrap())
                        .expect("Error deserializing key");
                self.reminders.remove(&key);
                self.selected = None;
                manager.remove_reminder(message.channel_id, key).await;
            }
            _ => (),
        }

        if let Err(why) = message
            .create_interaction_response(&ctx.http, move |response| {
                response
                    .kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|message| self.create(message))
            })
            .await
        {
            println!("Cannot respond to component interaction: {:#?}", why);
        }
    }
}
