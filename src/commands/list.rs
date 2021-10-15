use super::*;
use crate::reminder_menu::ReminderMenu;

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
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        _options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
        manager
            .channel_data(command.channel_id)
            .await
            .filter(|cd| cd.reminders.len() > 0)
            .map(|cd| {
                cd.reminders
                    .iter()
                    .map(|(_k, r)| r.sched.to_string() + " | " + &r.msg)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or("no reminders".to_string());

        let menu = ReminderMenu::new(&manager, command.channel_id).await;
        if let Err(why) = command
            .create_interaction_response(&ctx.http, move |response| {
                response.interaction_response_data(|command| menu.create(command))
            })
            .await
        {
            println!("Cannot respond to slash command: {:#?}", why);
        }
    }
}
