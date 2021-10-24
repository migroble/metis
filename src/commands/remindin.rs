use super::*;
use crate::reminder::{Reminder, ReminderType};
use chrono::{offset::Utc, Duration};

pub struct RemindIn;

#[async_trait]
impl Command for RemindIn {
    fn name(&self) -> &str {
        "remindin"
    }

    fn create(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Sends delayed message")
            .create_option(|option| {
                option
                    .name("msg")
                    .description("Message to be sent")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("mins")
                    .description("Minutes")
                    .kind(ApplicationCommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("hours")
                    .description("Hours")
                    .kind(ApplicationCommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("days")
                    .description("Days")
                    .kind(ApplicationCommandOptionType::Integer)
                    .required(false)
            });
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        let msg = if let ApplicationCommandInteractionDataOptionValue::String(s) =
            options.get("msg").unwrap()
        {
            s
        } else {
            panic!("Expected message to be string")
        }
        .to_string();
        let options = options
            .into_iter()
            .filter_map(|(k, v)| {
                if let ApplicationCommandInteractionDataOptionValue::Integer(i) = v {
                    Some((k, i))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        let get = move |name: &str| *options.get(name).unwrap_or(&0);

        // Calculate the datetime the reminder must be sent at
        let delay = Duration::minutes(get("mins") + 60 * (get("hours") + 24 * get("days")));
        let later = Utc::now() + delay;

        manager
            .add_reminder(
                Arc::clone(&ctx),
                command.channel_id,
                Reminder {
                    reminder_type: ReminderType::Once(later.naive_utc()),
                    msg,
                },
            )
            .await;

        if let Err(why) = command
            .create_interaction_response(&ctx.http, move |response| {
                response.interaction_response_data(|message| message.content("done"))
            })
            .await
        {
            println!("Cannot respond to slash command: {:#?}", why);
        }
    }
}
