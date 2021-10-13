use crate::manager::Manager;
use crate::reminder::Reminder;
use chrono::Utc;
use chrono_tz::{Etc::UTC, Tz};
use cron::Schedule;
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::{ChannelId, GuildId},
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
use slotmap::DefaultKey;
use std::{str::FromStr, sync::Arc};
use tokio::{sync::RwLock, time::sleep};

// DEV DEP: Used to get DEV_GUILD
use std::env;

pub struct Handler {
    db: Arc<RwLock<Manager>>,
}

impl Handler {
    pub async fn with_file(db_path: &str) -> Self {
        Self {
            db: Arc::new(RwLock::new(Manager::open(db_path).await)),
        }
    }

    fn start_reminding(
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

    async fn add_reminder(&self, ctx: Arc<Context>, channel_id: ChannelId, reminder: Reminder) {
        let key = self
            .db
            .write()
            .await
            .insert(channel_id, reminder.clone())
            .await;
        self.start_reminding(ctx, channel_id, UTC, key, reminder);
    }

    async fn remove_reminder(&self, channel_id: ChannelId, key: DefaultKey) {
        self.db.write().await.remove(channel_id, key).await;
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let ctx = Arc::new(ctx);

            // TODO: Add list and remove reminder commands (explore using action row -> list
            // + delete button)

            // TODO: Add tempremindme command that creates an action row with a button to
            // stop the reminders
            let content = match command.data.name.as_str() {
                "remindme" => {
                    let mut options: Vec<String> = command
                        .data
                        .options
                        .iter()
                        .map(|o| {
                            // This should never panic
                            if let ApplicationCommandInteractionDataOptionValue::String(s) =
                                o.resolved.as_ref().expect("Expected option")
                            {
                                s
                            } else {
                                panic!("Expected string option");
                            }
                        })
                        .cloned()
                        .collect();

                    // This shouldn't panic either
                    let msg = options.pop().expect("Expected message");

                    let sched = "0 ".to_string() + &options.join(" ") + " *";
                    if let Ok(sched) = Schedule::from_str(&sched) {
                        self.add_reminder(
                            Arc::clone(&ctx),
                            command.channel_id,
                            Reminder { sched, msg },
                        )
                        .await;

                        "done"
                    } else {
                        "invalid cron expression"
                    }
                    .to_string()
                }
                "list" => {
                    let reminders = self
                        .db
                        .read()
                        .await
                        .channel_iter(command.channel_id)
                        .map_or_else(Vec::new, |i| {
                            i.map(|(_k, r)| r.sched.to_string() + " | " + &r.msg)
                                .collect::<Vec<_>>()
                        });

                    reminders.join("\n")
                }
                "tz" => {
                    let option = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected option")
                        .resolved
                        .as_ref()
                        .expect("Expected string");
                    let tz_str =
                        if let ApplicationCommandInteractionDataOptionValue::String(s) = option {
                            s
                        } else {
                            panic!("Expected string option");
                        };

                    if self
                        .db
                        .write()
                        .await
                        .set_tz(command.channel_id, tz_str)
                        .await
                        .is_ok()
                    {
                        "done"
                    } else {
                        "invalid timezone (list of timezone names: <https://w.wiki/4Jx>, capitalization matters!)"
                    }
                    .to_string()
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
        self.db
            .read()
            .await
            .iter()
            .for_each(|(c, t, k, r)| self.start_reminding(Arc::clone(&ctx), *c, t, k, r.clone()));

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
                                .name("min")
                                .description("Minute (0-59)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("hour")
                                .description("Hour (0-23)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("dom")
                                .description("Day of month (1-31)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("month")
                                .description("Month (1-12 or Jan-Dec)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("dow")
                                .description("Day of week (Sun-Sat)")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("msg")
                                .description("Message to be sent")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
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
