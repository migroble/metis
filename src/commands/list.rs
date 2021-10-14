use crate::{command_handler::CommandHandler, manager::Manager};
use serenity::{
    async_trait,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};

pub struct List;

#[async_trait]
impl CommandHandler for List {
    fn can_handle(&self, name: String) -> bool {
        name == "list"
    }

    async fn handle(
        &self,
        _ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        _options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) -> String {
        manager
            .channel_data(command.channel_id)
            .await
            .map(|cd| {
                cd.reminders
                    .iter()
                    .map(|(_k, r)| r.sched.to_string() + " | " + &r.msg)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or("no reminders".to_string())
    }
}
