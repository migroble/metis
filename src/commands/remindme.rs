use crate::{command_handler::CommandHandler, manager::Manager, reminder::Reminder};
use chrono::{Datelike, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use serenity::{
    async_trait,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
    prelude::*,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};

pub struct RemindMe;

#[async_trait]
impl CommandHandler for RemindMe {
    fn can_handle(&self, name: &str) -> bool {
        name == "remindme"
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) -> String {
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
                (now.month().to_string(), now.weekday().to_string())
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
        if let Ok(sched) = Schedule::from_str(&sched) {
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
        .to_string()
    }
}
