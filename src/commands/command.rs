use super::*;

#[async_trait]
pub trait Command {
    fn name(&self) -> &str;

    fn create(&self, command: &mut CreateApplicationCommand);

    async fn handle(
        &self,
        ctx: Arc<Context>,
        manager: &Manager,
        command: &ApplicationCommandInteraction,
        options: HashMap<String, ApplicationCommandInteractionDataOptionValue>,
    );
}
