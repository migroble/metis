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
}

impl Handler {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            manager: Manager::with_file(db_path).await,
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
            let content = match command.data.name.as_str() {
                "remindme" => {
                    RemindMe
                        .handle(Arc::clone(&ctx), &self.manager, &command, options)
                        .await
                }
                "list" => {
                    List.handle(Arc::clone(&ctx), &self.manager, &command, options)
                        .await
                }
                "tz" => {
                    Tz.handle(Arc::clone(&ctx), &self.manager, &command, options)
                        .await
                }
                _ => "not implemented".to_string(),
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
            commands
                .create_application_command(|command| {
                    command
                        .name("remindme")
                        .description("Sends message at scheduled time(s) using cron format")
                        .create_option(|option| {
                            option
                                .name("msg")
                                .description("Message to be sent")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("min")
                                .description("Minute (0-59)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                        .create_option(|option| {
                            option
                                .name("hour")
                                .description("Hour (0-23)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                        .create_option(|option| {
                            option
                                .name("dom")
                                .description("Day of month (1-31)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                        .create_option(|option| {
                            option
                                .name("month")
                                .description("Month (1-12 or Jan-Dec)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                        .create_option(|option| {
                            option
                                .name("dow")
                                .description("Day of week (Sun-Sat)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                        .create_option(|option| {
                            option
                                .name("year")
                                .description("Year")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("list")
                        .description("Lists all reminders for this channel")
                })
                .create_application_command(|command| {
                    command
                        .name("tz")
                        .description("Set timezone for this channel")
                        .create_option(|option| {
                            option
                                .name("tz")
                                .description("IANA timezone name")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
        })
        .await
        .expect("Error creating commands");
    }
}
