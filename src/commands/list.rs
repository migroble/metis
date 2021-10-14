use super::*;

pub struct List;

#[async_trait]
impl Command for List {
    fn create(&self, command: &mut CreateApplicationCommand) {
        command
            .name("list")
            .description("Lists all reminders for this channel");
    }

    fn can_handle(&self, name: &str) -> bool {
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
