use super::*;
use crate::reminder_menu::ReminderMenu;

pub struct Menu;

#[async_trait]
impl Command for Menu {
    fn name(&self) -> &str {
        "menu"
    }

    fn create(&self, command: &mut CreateApplicationCommand) {
        command.description("Show all reminders for this channel");
    }

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        _options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    ) {
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
