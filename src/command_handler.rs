use crate::manager::Manager;
use serenity::{
    async_trait,
    model::interactions::application_command::{
        ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
    },
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait CommandHandler {
    fn can_handle(&self, name: &str) -> bool;
    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) -> String;
}
