use chrono::{DateTime, FixedOffset, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
};

pub mod dtos;
use dtos::*;

impl EventDto {
    fn to_string(&self) -> String {
        let time_left = self.end_time().to_utc() - Utc::now();
        format!(
            "{}({}):\n    start: {}\n    end: {}\n    time left: {}h\n    {} participants\n",
            self.title,
            self.shrinked_format(),
            self.start_time().format("%H:%M %v"),
            self.end_time().format("%H:%M %v"),
            time_left.num_hours(),
            self.participants
        )
    }
    fn shrinked_format(&self) -> String {
        match self.format.as_str() {
            "Attack-Defense" => "A/D".to_string(),
            "Jeopardy" => "J".to_string(),
            _ => self.format.clone(),
        }
    }
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

    let resp = reqwest::get(format!(
        "https://ctftime.org/api/v1/events/?limit=200&start={}&finish={}",
        week_ago_ts, week_later_ts
    ))
    .await?
    .json::<Vec<EventDto>>()
    .await?;

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send>> {
    pretty_env_logger::init();
    let bot = Bot::from_env();

    let handler = dptree::entry().branch(Update::filter_inline_query().endpoint(
        |bot: Bot, q: InlineQuery| async move {
            let api_resp = get_events().await.unwrap();
            let mut ongoing = api_resp
                .into_iter()
                .filter(|dto| dto.is_ongoing())
                .collect::<Vec<EventDto>>();
            ongoing.sort_by_key(|dto| dto.participants);
            ongoing.reverse();
            let query_response = ongoing
                .iter()
                .map(|dto| dto.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            let article = InlineQueryResultArticle::new(
                "0".to_string(),
                "Show ongoing",
                InputMessageContent::Text(InputMessageContentText::new(query_response)),
            );
            let response = bot
                .answer_inline_query(&q.id, vec![InlineQueryResult::Article(article)])
                .send()
                .await;
            if let Err(err) = response {
                log::error!("Error in handler: {:?}", err);
            }
            respond(())
        },
    ));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
