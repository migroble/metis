use super::*;

#[async_trait]
pub trait Command {
    fn create(&self, command: &mut CreateApplicationCommand);

    fn can_handle(&self, name: &str) -> bool;

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    );
}
