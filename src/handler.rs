use crate::{
    commands::{Command, Menu, RemindMe, RemindOnce, Tz},
    manager::Manager,
    reminder_menu::ReminderMenu,
};
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{application_command::ApplicationCommand, Interaction},
    },
    prelude::*,
};
use std::sync::Arc;

// DEV DEP: Used to get DEV_GUILD
use std::env;

pub struct Handler {
    manager: Manager,
    commands: Vec<&'static (dyn Command + Sync)>,
}

impl Handler {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            manager: Manager::with_file(db_path).await,
            commands: vec![&Menu, &RemindMe, &RemindOnce, &Tz],
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
                    .filter(|c| c.can_handle(&command.data.name))
                    .next();

                if let Some(c) = handler_opt {
                    c.handle(Arc::clone(&ctx), &self.manager, &command, options)
                        .await
                }
            }
            Interaction::MessageComponent(message) => {
                let mut menu = ReminderMenu::new(&self.manager, message.channel_id).await;
                menu.handle(Arc::clone(&ctx), &self.manager, &message).await;
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
        // ApplicationCommand::set_global_application_commands
        GuildId(
            env::var("DEV_GUILD")
                .expect("Expected dev guild ID in the environment")
                .parse()
                .expect("Failed to parse guild ID"),
        )
        .set_application_commands(&ctx.http, |commands| {
            self.commands.iter().fold(commands, |commands, c| {
                commands.create_application_command(|command| {
                    c.create(command);
                    command
                })
            })
        })
        .await
        .expect("Error creating commands");
    }
}
