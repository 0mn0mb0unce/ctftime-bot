use chrono::Utc;
use handlebars::{to_json, Handlebars};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::value::Map;

use crate::EventDto;

lazy_static! {
    pub static ref DT_FORMAT: String = String::from("%H:%M %v");
}

#[derive(Serialize)]
pub struct OngoingEventsTemplate {
    pub current_dt: String,
    pub events: Vec<EventTemplate>,
}

#[derive(Serialize)]
pub struct EventTemplate {
    title: String,
    start_dt: String,
    end_dt: String,
    hours_left: i64,
    pub participants: i64,
    format: String,
}

impl EventTemplate {
    pub fn from_dto(e: &EventDto) -> Self {
        Self {
            title: e.title.clone(),
            start_dt: e.start_time().format(&DT_FORMAT).to_string(),
            end_dt: e.end_time().format(&DT_FORMAT).to_string(),
            hours_left: (e.end_time().to_utc() - Utc::now()).num_hours(),
            participants: e.participants,
            format: e.format.clone(),
        }
    }
}

impl OngoingEventsTemplate {
    pub fn render(&self, handlebars: &Handlebars) -> String {
        let mut data = Map::new();
        data.insert("data".to_string(), to_json(self));
        handlebars
            .render("ongoing", &data)
            .unwrap()
            .replace("-", "\\-")
            .replace("(", "\\(")
            .replace(")", "\\)")
    }
}
