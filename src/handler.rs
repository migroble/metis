use crate::{
    commands::{Command, Menu, RemindIn, RemindMe, RemindOnce, Tz},
    manager::Manager,
    reminder::{Reminder, ReminderType},
    reminder_menu::ReminderMenu,
};
use chrono::{Duration, Utc};
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{
            application_command::ApplicationCommand, Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::sync::Arc;

pub struct Handler {
    manager: Manager,
    commands: Vec<&'static (dyn Command + Sync)>,
}

impl Handler {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            manager: Manager::with_file(db_path).await,
            commands: vec![&Menu, &RemindIn, &RemindMe, &RemindOnce, &Tz],
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let ctx = Arc::new(ctx);

        match interaction {
            Interaction::ApplicationCommand(command) => {
                let options = command
                    .data
                    .options
                    .iter()
                    .map(|o| {
                        (
                            o.name.clone(),
                            o.resolved.as_ref().expect("Expected option").clone(),
                        )
                    })
                    .collect();

                // TODO: Add tempremindme command that creates an action row with a button to
                // stop the reminders
                let handler_opt = self
                    .commands
                    .iter()
                    .filter(|c| c.name() == command.data.name)
                    .next();

                if let Some(c) = handler_opt {
                    c.handle(Arc::clone(&ctx), &self.manager, &command, options)
                        .await
                }
            }
            Interaction::MessageComponent(message) => {
                let mut parts = message.data.custom_id.splitn(2, "-");
                if let Some(prefix) = parts.next() {
                    match prefix {
                        "menu" => {
                            let mut menu =
                                ReminderMenu::new(&self.manager, message.channel_id).await;
                            menu.handle(Arc::clone(&ctx), &self.manager, &message).await;
                        }
                        "postpone" => {
                            let dt = parts.next().unwrap().parse().unwrap();
                            let msg = message.message.content.clone();

                            let reminder = Reminder {
                                reminder_type: ReminderType::Once(
                                    Utc::now().naive_utc() + Duration::minutes(dt),
                                ),
                                msg,
                            };

                            self.manager
                                .add_reminder(Arc::clone(&ctx), message.channel_id, reminder)
                                .await;

                            if let Err(why) = message
                                .create_interaction_response(&ctx.http, move |response| {
                                    response
                                        .kind(InteractionResponseType::UpdateMessage)
                                        .interaction_response_data(|message| message)
                                })
                                .await
                            {
                                println!("Cannot respond to component interaction: {:#?}", why);
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);

        // Start reminders
        self.manager.start_reminders(Arc::clone(&ctx)).await;

        // Set commands up
        ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            self.commands.iter().fold(commands, |commands, c| {
                commands.create_application_command(|command| {
                    c.create(command.name(c.name()));
                    command
                })
            })
        })
        .await
        .expect("Error creating commands");
    }
}
