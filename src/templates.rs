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

#[derive(Serialize)]
pub struct EventDetailedTemplate {
    organizers: Vec<String>,
    ct_url: String,
    url: String,
    weight: f64,
    duration: String,
    title: String,
    start_dt: String,
    end_dt: String,
    participants: i64,
    location: String,
    description: String,
    format: String,
    is_votable_now: bool,
    public_votable: bool,
    prizes: String,
    onsite: bool,
    restrictions: String,
}

impl EventDetailedTemplate {
    pub fn from_dto(dto: &EventDto) -> Self {
        Self {
            organizers: dto
                .organizers
                .iter()
                .map(|org| org.name.clone())
                .collect::<Vec<String>>(),
            ct_url: dto.ctftime_url.clone(),
            url: dto.url.clone(),
            weight: dto.weight,
            duration: dto.duration.pretty_print(),
            title: dto.title.clone(),
            start_dt: dto.start_time().format(&DT_FORMAT).to_string(),
            end_dt: dto.end_time().format(&DT_FORMAT).to_string(),
            participants: dto.participants,
            location: dto.location.clone(),
            description: dto.description.clone(),
            format: dto.format.clone(),
            is_votable_now: dto.is_votable_now,
            public_votable: dto.public_votable,
            prizes: dto.prizes.clone(),
            onsite: dto.onsite,
            restrictions: dto.restrictions.clone(),
        }
    }
    pub fn render(data: &Option<Self>, handlebars: &Handlebars) -> String {
        let mut template_data = Map::new();
        template_data.insert("event".to_string(), to_json(data));
        handlebars.render("event", &template_data).unwrap()
    }
}

impl OngoingEventsTemplate {
    pub fn render(&self, handlebars: &Handlebars) -> String {
        let mut data = Map::new();
        data.insert("data".to_string(), to_json(self));
        handlebars.render("ongoing", &data).unwrap()
    }
}
