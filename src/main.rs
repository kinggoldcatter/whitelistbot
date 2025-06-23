use http2byond::{ByondTopicValue, send_byond};
use poise::serenity_prelude::{self as serenity, RoleId};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::fs::{self, File, read_to_string};
use std::io::{Read, Write};
use std::net::SocketAddr;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const BOT_SOKET: ([u8; 4], u16) = ([127, 0, 0, 1], 5829);

/// Displays your or another user's account creation date
#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "ADMINISTRATOR"
)]
async fn whitelist(
    ctx: Context<'_>,
    #[description = "Ckey of the whitelist to be"] ckey: String,
    #[description = "Discord account to be given Resident"] user: Option<serenity::User>,
) -> Result<(), Error> {
    if let Some(user) = user {
        ctx.http()
            .add_member_role(
                ctx.guild_id().expect("this bot is only for servers"),
                user.id,
                RoleId::from(1344838322257793074),
                Some("Added to whitelist."),
            )
            .await
            .expect("attempted to add role to already recident");
    }

    let current_ckeys: HashSet<String> = load_white_list();
    let mut file = OpenOptions::new()
        .append(true)
        .open(std::env::var("WL_LOCATION").unwrap())
        .unwrap();

    let response: String;

    if current_ckeys.contains(&ckey) {
        response = format!("{} is already accepted by Astrata!", &ckey);
    } else {
        if let Err(e) = writeln!(file, "{ckey}") {
            eprintln!("Error adding ckey {ckey} : {e}");
            response = format!("Error adding ckey {ckey} : {e}");
        } else {
            response = format!("{ckey} has been added to Astrata's list of residents");
        }

        update_wl();
    }
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "ADMINISTRATOR"
)]
async fn blacklist(
    ctx: Context<'_>,
    #[description = "Ckey of the person to be blacklisted"] ckey: String,
) -> Result<(), Error> {
    let mut current_ckeys: HashSet<String> = load_white_list();
    let response: String = if current_ckeys.contains(&ckey) {
        current_ckeys.remove(&ckey);
        let mut new_ckey_list = String::from("#WHITELIST FOR LYNDVHAR\n\n");

        for ckey in current_ckeys {
            if !ckey.is_empty() {
                let _ = writeln!(new_ckey_list, "{ckey}");
            }
        }

        _ = fs::remove_file(std::env::var("WL_LOCATION").unwrap());

        _ = fs::write(std::env::var("WL_LOCATION").unwrap(), &new_ckey_list);

        format!("{ckey} has been banished back to ZIZO")
    } else {
        format!("{ckey} is already not welcome")
    };
    update_wl();
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    default_member_permissions = "ADMINISTRATOR"
)]
async fn print_wl(ctx: Context<'_>) -> Result<(), Error> {
    let response = read_to_string(std::env::var("WL_LOCATION").unwrap()).unwrap();
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn check_status(
    ctx: Context<'_>,
    #[description = "Ckey of the person to check"] ckey: String,
) -> Result<(), Error> {
    let current_ckeys: HashSet<String> = load_white_list();
    let response: String = if current_ckeys.contains(&ckey) {
        format!("{ckey} is welcome")
    } else {
        format!("{ckey} is not welcome")
    };
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn pingserver(ctx: Context<'_>) -> Result<(), Error> {
    let mut response = String::from("bingus");
    match send_byond(&SocketAddr::from(BOT_SOKET), "?playing") {
        Err(_) => {}
        Ok(btv_result) => match btv_result {
            ByondTopicValue::None => response = "Byond returned nothing".to_string(),
            ByondTopicValue::String(str) => response = format!("Byond returned string {str}"),
            ByondTopicValue::Number(num) => response = format!("There are currently {num} at the table."),
        },
    }

    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                whitelist(),
                blacklist(),
                pingserver(),
                print_wl(),
                check_status(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

fn load_white_list() -> std::collections::HashSet<std::string::String> {
    let mut file =
        File::open(std::env::var("WL_LOCATION").unwrap()).expect("there will always be a file");

    let mut current_wl: HashSet<String> = HashSet::new();

    let mut ckeys = String::new();
    file.read_to_string(&mut ckeys).unwrap();

    let ckeys = ckeys.split('\n');

    for line in ckeys.skip(2) {
        current_wl.insert(line.to_string());
    }

    current_wl
}

fn update_wl() {
    match send_byond(&SocketAddr::from(BOT_SOKET), "?updatewl") {
        Err(_) => {}
        Ok(btv_result) => match btv_result {
            ByondTopicValue::None => println!("whitelist updated"),
            ByondTopicValue::String(str) => println!("Byond returned string {str}"),
            ByondTopicValue::Number(num) => println!("Byond returned number {num}"),
        },
    }
}
