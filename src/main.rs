#![feature(duration_millis_float)]

use chrono::{DateTime, FixedOffset, Utc};
use dptree::case;
use handlebars::Handlebars;
use std::time::{Duration, Instant};
use std::{sync::Arc, time::SystemTime};
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
    utils::command::BotCommands,
};

pub mod api;
pub mod dtos;
pub mod metrics;
pub mod templates;
use api::*;
use dtos::*;
use metrics::*;
use templates::*;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

impl EventDto {
    fn start_time(&self) -> DateTime<FixedOffset> {
        let tz = FixedOffset::east_opt(3 * 60 * 60).unwrap();
        DateTime::parse_from_rfc3339(&self.start)
            .unwrap()
            .with_timezone(&tz)
    }
    fn end_time(&self) -> DateTime<FixedOffset> {
        let tz = FixedOffset::east_opt(3 * 60 * 60).unwrap();
        DateTime::parse_from_rfc3339(&self.finish)
            .unwrap()
            .with_timezone(&tz)
    }
    fn is_ongoing(&self) -> bool {
        let current_time = Utc::now();
        current_time > self.start_time() && current_time < self.end_time()
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum BotCommand {
    #[command(aliases = ["h"], description = "show this help message")]
    Help,
    #[command(aliases = ["e"], description = "show information about event by id")]
    Event(u64),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    pretty_env_logger::init();
    let bot = Bot::from_env();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("ongoing", include_str!("templates/ongoing.hbs"))
        .unwrap();
    handlebars
        .register_template_string("event", include_str!("templates/event.hbs"))
        .unwrap();

    let handler = dptree::entry()
        .branch(Update::filter_inline_query().endpoint(handle_inline_query))
        .branch(
            Update::filter_message().branch(
                teloxide::filter_command::<BotCommand, _>()
                    .branch(case![BotCommand::Help].endpoint(handle_help_command))
                    .branch(case![BotCommand::Event(id)].endpoint(handle_get_event_command)),
            ),
        );

    tokio::task::spawn(run_metrics_server());

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![Arc::new(handlebars)])
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_help_command(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .await?;
    Ok(())
}

async fn handle_get_event_command(
    bot: Bot,
    msg: Message,
    cmd: BotCommand,
    handlebars: Arc<Handlebars<'_>>,
) -> HandlerResult {
    let BotCommand::Event(event_id) = cmd else {
        return Ok(());
    };

    let api_resp = ctftime_api::get_event_by_id(event_id).await.unwrap();
    let template = api_resp.map(|x| EventDetailedTemplate::from_dto(&x));
    let query_response = EventDetailedTemplate::render(&template, &handlebars);
    bot.send_message(msg.chat.id, query_response)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

async fn handle_inline_query(
    bot: Bot,
    q: InlineQuery,
    handlebars: Arc<Handlebars<'_>>,
) -> HandlerResult {
    INCOMING_REQUESTS.with_label_values(&["ongoing"]).inc();
    let start_handling_instant = Instant::now();

    let current_time = SystemTime::now();
    let week_ago = current_time
        .checked_sub(Duration::from_secs(60 * 60 * 24 * 7))
        .unwrap();
    let week_later = current_time
        .checked_add(Duration::from_secs(60 * 60 * 24 * 7))
        .unwrap();
    let api_resp = ctftime_api::get_events(week_ago, week_later).await.unwrap();
    let mut ongoing = api_resp
        .into_iter()
        .filter(|dto| dto.is_ongoing())
        .map(|dto| EventTemplate::from_dto(&dto))
        .collect::<Vec<EventTemplate>>();
    ongoing.sort_by_key(|tmpl| tmpl.participants);
    ongoing.reverse();

    let template = OngoingEventsTemplate {
        current_dt: Utc::now().format(&DT_FORMAT).to_string(),
        events: ongoing,
    };
    let query_response = template.render(&handlebars);

    let article = InlineQueryResultArticle::new(
        "0".to_string(),
        "Show ongoing",
        InputMessageContent::Text(
            InputMessageContentText::new(query_response)
                .parse_mode(teloxide::types::ParseMode::Html),
        ),
    );
    let response = bot
        .answer_inline_query(&q.id, vec![InlineQueryResult::Article(article)])
        .cache_time(60)
        .send()
        .await;
    if let Err(err) = response {
        log::error!("Error in handler: {:?}", err);
    }

    let elapsed_time = start_handling_instant.elapsed().as_millis_f64();
    RESPONSE_TIME_COLLECTOR
        .with_label_values(&["ongoing"])
        .observe(elapsed_time / 1000_f64);

    Ok(())
}
