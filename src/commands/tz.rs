use crate::{command_handler::CommandHandler, manager::Manager};
use serenity::{
    async_trait,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};

pub struct Tz;

#[async_trait]
impl CommandHandler for Tz {
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
