# Metis
> An assistant Discord bot

Tired of forgetting things? Let Metis remind you! [Invite Metis to your server](https://discord.com/api/oauth2/authorize?client_id=896183942372290611&permissions=2147485696&scope=bot%20applications.commands).

Metis is a Discord bot that lets you add custom messages to be sent in the same channel on a schedule or as one-offs. This leverages Discord as a cross-platform notification system. 

While it is recommeneded you only use Metis in direct messages, you need to share a server to be able to message it.


## Features

* One-off reminders
* Scheduled reminders
* Delayed reminders
* [Slash commands](https://discord.com/developers/docs/interactions/application-commands)
* Reminder management
* Per-channel reminders
* cron-like syntax for scheduled and one-off reminders

## Commands

* `/remindme`: Creates a scheduled reminder
* `/remindonce`: Creates a one-off reminder
* `/remindin`: Creates a one-off reminder after a delay
* `/menu`: Shows a list of reminders, allows you to select and delete them
* `/tz`: Sets the current channel's timezone (case-sensitive [IANA timezone name](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones))

## Hosting your own instance

1. [Set up a discord application with a bot](https://discord.com/developers/docs/intro#bots-and-apps).
2. Build the project directly with `cargo build --release` or as a Docker image with `docker build .`.
3. Set the following environment variables or put them in a `.env` file in the same directory as your executable:
   * `APPLICATION_ID` & `DISCORD_TOKEN`: The bot's application ID and token you got in step 1.
   * `DB_FILE`: The path to the file where the reminders are stored. If you are running the bot in a container it is recommended you use a [volume](https://docs.docker.com/storage/volumes/).
   * `DEV_GUILD`: The ID of the channel where the commands are set up. This variable is only for [development purposes](https://docs.rs/serenity/0.10.9/serenity/model/interactions/application_command/struct.ApplicationCommand.html#method.create_global_application_command) and will be removed in the future.
4. Run the executable (should be in target/release) or instantiate the image with `docker run --env-file .env <image id>`.
5. Invite your bot to your server.
6. Done!

## What's with the name?

[Metis](https://en.wikipedia.org/wiki/Metis_(mythology)) is an ancient Greek goddess, mother of wisdom and deep thought, so it stands to reason she would remind you of things. Maybe that's a little contrived...
