#![feature(duration_millis_float)]

use chrono::{DateTime, FixedOffset, Utc};
use handlebars::Handlebars;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
};

pub mod dtos;
pub mod metrics;
pub mod templates;
use dtos::*;
use metrics::*;
use templates::*;

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

async fn get_events() -> Result<Vec<EventDto>, Box<dyn std::error::Error>> {
    let current_time = SystemTime::now();
    let week_ago = current_time
        .checked_sub(Duration::from_secs(60 * 60 * 24 * 7))
        .unwrap();
    let week_later = current_time
        .checked_add(Duration::from_secs(60 * 60 * 24 * 7))
        .unwrap();

    let week_ago_ts = week_ago.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let week_later_ts = week_later.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let url = format!(
        "https://ctftime.org/api/v1/events/?limit=200&start={}&finish={}",
        week_ago_ts, week_later_ts
    );
    let resp = reqwest::get(url).await?.json::<Vec<EventDto>>().await?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    pretty_env_logger::init();
    let bot = Bot::from_env();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("ongoing", include_str!("templates/ongoing.hbs"))
        .unwrap();

    let handler =
        dptree::entry().branch(Update::filter_inline_query().endpoint(handle_inline_query));

    tokio::task::spawn(run_metrics_server());

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![Arc::new(handlebars)])
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_inline_query(
    bot: Bot,
    q: InlineQuery,
    handlebars: Arc<Handlebars<'_>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    INCOMING_REQUESTS.with_label_values(&["ongoing"]).inc();
    let start_handling_instant = Instant::now();

    let api_resp = get_events().await.unwrap();
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
                .parse_mode(teloxide::types::ParseMode::MarkdownV2),
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
