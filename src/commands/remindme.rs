use super::*;
use crate::reminder::Reminder;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use std::str::FromStr;

pub struct RemindMe;

#[async_trait]
impl Command for RemindMe {
    fn create(&self, command: &mut CreateApplicationCommand) {
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
            });
    }

    fn can_handle(&self, name: &str) -> bool {
        name == "remindme"
    }

    async fn handle(
        &self,
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

        // We get the timezone-adjusted current datetime as fallback values
        let tz = manager
            .channel_tz(command.channel_id)
            .await
            .unwrap_or(Tz::Etc__UTC);
        let now = tz.from_utc_datetime(&Utc::now().naive_utc());

        // Day of month and day of week are interrelated, therefore we must be careful
        // when using them as fallback values
        //
        // If only one is set, we should set the other to "?" rather than the current
        // date to avoid unintended behaviour
        let (dom, dow) = {
            let dom_opt = options.get("dom");
            let dow_opt = options.get("dow");

            if let Some(dom) = dom_opt {
                if let Some(dow) = dow_opt {
                    (dom.to_string(), dow.to_string())
                } else {
                    (dom.to_string(), "?".to_string())
                }
            } else if let Some(dow) = dow_opt {
                ("?".to_string(), dow.to_string())
            } else {
                (now.day().to_string(), now.weekday().to_string())
            }
        };

        // We always put a 0 in the seconds slot since it is unlikely to be useful to
        // the end user
        let sched = format!(
            "0 {} {} {} {} {} {}",
            options.get("min").unwrap_or(&now.minute().to_string()),
            options.get("hour").unwrap_or(&now.hour().to_string()),
            dom,
            options.get("month").unwrap_or(&now.month().to_string()),
            dow,
            options.get("year").unwrap_or(&now.year().to_string()),
        );

        println!("{}", sched);
        let content = if let Ok(sched) = Schedule::from_str(&sched) {
            // The msg option is required, we are guaranteed to have it
            let msg = options.get("msg").unwrap().to_string();

            manager
                .add_reminder(
                    Arc::clone(&ctx),
                    command.channel_id,
                    Reminder { sched, msg },
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
