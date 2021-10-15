use super::*;

pub struct Tz;

#[async_trait]
impl Command for Tz {
    fn name(&self) -> &str {
        "tz"
    }

    fn create(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Set timezone for this channel")
            .create_option(|option| {
                option
                    .name("tz")
                    .description("IANA timezone name")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            });
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        let tz_str = if let ApplicationCommandInteractionDataOptionValue::String(s) = &options["tz"]
        {
            s
        } else {
            panic!("Expected string option");
        };

        let content = if manager.set_channel_tz(command.channel_id, &tz_str)
            .await
            .is_ok()
        {
            "done"
        } else {
            "invalid timezone (list of timezone names: <https://w.wiki/4Jx>, capitalization matters!)"
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
