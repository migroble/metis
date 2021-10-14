use super::*;

pub struct Tz;

#[async_trait]
impl Command for Tz {
    fn create(&self, command: &mut CreateApplicationCommand) {
        command
            .name("tz")
            .description("Set timezone for this channel")
            .create_option(|option| {
                option
                    .name("tz")
                    .description("IANA timezone name")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            });
    }

    fn can_handle(&self, name: &str) -> bool {
        name == "tz"
    }

    async fn handle(
        &self,
        _ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) -> String {
        let tz_str = if let ApplicationCommandInteractionDataOptionValue::String(s) = &options["tz"]
        {
            s
        } else {
            panic!("Expected string option");
        };

        if manager.set_channel_tz(command.channel_id, &tz_str)
            .await
            .is_ok()
        {
            "done"
        } else {
            "invalid timezone (list of timezone names: <https://w.wiki/4Jx>, capitalization matters!)"
        }
        .to_string()
    }
}
