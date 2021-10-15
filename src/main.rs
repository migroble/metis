#![deny(clippy::pedantic)]

mod commands;
mod db;
mod handler;
mod manager;
mod reminder;
mod reminder_menu;

use dotenv::dotenv;
use handler::Handler;
use serenity::prelude::*;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = env::var("APPLICATION_ID")
        .expect("Expected an application ID in the environment")
        .parse()
        .expect("Application ID is not a number");

    let mut client = Client::builder(token)
        .event_handler(
            Handler::with_file(
                &env::var("DB_FILE").expect("Expected database file path in environment"),
            )
            .await,
        )
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
