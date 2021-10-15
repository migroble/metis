use crate::{
    manager::Manager,
    reminder::{ChannelData, Reminder},
};
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
                                sm.custom_id("reminders").options(|opts| {
                                    self.reminders.iter().fold(opts, |opts, (k, r)| {
                                        opts.create_option(|opt| {
                                            // This should never panic, any key should be
                                            // stringifiable
                                            let key = serde_json::to_string(k)
                                                .expect("Error serializing key");
                                            opt.label(&r.msg)
                                                .description(
                                                    r.sched
                                                        .upcoming(self.tz)
                                                        .next()
                                                        .map(|t| t.to_rfc2822())
                                                        .unwrap_or("".to_string()),
                                                )
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
                                b.style(ButtonStyle::Secondary)
                                    .label("Done")
                                    .custom_id("done")
                            })
                            .create_button(|b| {
                                let b = b.style(ButtonStyle::Danger).label("Delete");
                                if let Some(sel) = &self.selected {
                                    b.custom_id(&sel)
                                } else {
                                    b.custom_id("delete").disabled(true)
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
                if message.data.custom_id == "done" {
                    if let Err(why) = message
                        .create_interaction_response(&ctx.http, move |response| {
                            response
                                .kind(InteractionResponseType::UpdateMessage)
                                .interaction_response_data(|message| {
                                    message.content("done").components(|comps| comps)
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to component interaction: {:#?}", why);
                    }
                    return;
                }

                // This should never panic because the custom_id is always the valid json string
                // we generated in ReminderMenu::create
                let key =
                    serde_json::from_str(&message.data.custom_id).expect("Error deserializing key");
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
