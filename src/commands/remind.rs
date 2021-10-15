use super::*;
use crate::reminder::{Reminder, ReminderType};
use chrono_tz::Etc::UTC;
use cron::Schedule;
use std::str::FromStr;

struct Remind;

impl Remind {
    fn create(command: &mut CreateApplicationCommand, description: &str) {
        command
            .description(description)
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
            });
    }

    async fn handle(
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        let options = options
            .into_iter()
            .map(|(k, v)| {
                if let ApplicationCommandInteractionDataOptionValue::String(s) = v {
                    (k, s.clone())
                } else {
                    panic!("Expected string option");
                }
            })
            .collect::<HashMap<_, _>>();

        // cron string construction

        // Day of month and day of week are interrelated, therefore we must be careful
        // when using them as fallback values
        //
        // If only one is set, we should set the other to "?"
        let question_mark = "?".to_string();
        let asterisk = "*".to_string();
        let (dom, dow) = {
            let dom_opt = options.get("dom");
            let dow_opt = options.get("dow");

            if let Some(dom) = dom_opt {
                if let Some(dow) = dow_opt {
                    (dom, dow)
                } else {
                    (dom, &question_mark)
                }
            } else if let Some(dow) = dow_opt {
                (&question_mark, dow)
            } else {
                (&asterisk, &asterisk)
            }
        };

        // We always put a 0 in the seconds slot since it is unlikely to be useful to
        // the end user
        let sched = format!(
            "0 {} {} {} {} {} {}",
            options.get("min").unwrap_or(&asterisk),
            options.get("hour").unwrap_or(&asterisk),
            dom,
            options.get("month").unwrap_or(&asterisk),
            dow,
            options.get("year").unwrap_or(&asterisk),
        );

        let content = if let Ok(sched) = Schedule::from_str(&sched) {
            // The msg option is required, we are guaranteed to have it
            let msg = options.get("msg").unwrap().to_string();

            let reminder_type = if command.data.name == "remindonce" {
                ReminderType::Once(
                    sched
                        .upcoming(manager.channel_tz(command.channel_id).await.unwrap_or(UTC))
                        .next()
                        .expect("Invalid schedule")
                        .naive_utc(),
                )
            } else {
                ReminderType::Scheduled(sched)
            };

            manager
                .add_reminder(
                    Arc::clone(&ctx),
                    command.channel_id,
                    Reminder { reminder_type, msg },
                )
                .await;

            "done"
        } else {
            "invalid cron expression"
        }
        .to_string();

        if let Err(why) = command
            .create_interaction_response(&ctx.http, move |response| {
                response.interaction_response_data(|message| message.content(content))
            })
            .await
        {
            println!("Cannot respond to slash command: {:#?}", why);
        }
    }
}

pub struct RemindMe;

#[async_trait]
impl Command for RemindMe {
    fn name(&self) -> &str {
        "remindme"
    }

    fn create(&self, command: &mut CreateApplicationCommand) {
        Remind::create(
            command,
            "Sends message at scheduled time(s) using cron format",
        );
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        Remind::handle(ctx, manager, command, options).await;
    }
}

pub struct RemindOnce;

#[async_trait]
impl Command for RemindOnce {
    fn name(&self) -> &'static str {
        "remindonce"
    }

    fn create(&self, command: &mut CreateApplicationCommand) {
        Remind::create(
            command,
            "Sends message once at a scheduled time using cron format",
        );
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        Remind::handle(ctx, manager, command, options).await;
    }
}
