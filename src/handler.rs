use crate::{
    command_handler::CommandHandler,
    commands::{List, RemindMe, Tz},
    manager::Manager,
};
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::sync::Arc;

// DEV DEP: Used to get DEV_GUILD
use std::env;

pub struct Handler {
    manager: Manager,
    commands: Vec<&'static (dyn CommandHandler + Sync)>,
}

impl Handler {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            manager: Manager::with_file(db_path).await,
            commands: vec![&RemindMe, &List, &Tz],
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let ctx = Arc::new(ctx);
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

            // TODO: Add list and remove reminder commands (explore using action row -> list
            // + delete button)

            // TODO: Add tempremindme command that creates an action row with a button to
            // stop the reminders
            let handler_opt = self
                .commands
                .iter()
                .filter(|c| c.can_handle(&command.data.name))
                .next();

            let content = if let Some(c) = handler_opt {
                c.handle(Arc::clone(&ctx), &self.manager, &command, options)
                    .await
            } else {
                "not_implemented".to_string()
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(content)
                            // .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                        })
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
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
