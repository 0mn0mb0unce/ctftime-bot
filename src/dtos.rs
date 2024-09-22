use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDto {
    pub organizers: Vec<OrganizerDto>,
    #[serde(rename = "ctftime_url")]
    pub ctftime_url: String,
    #[serde(rename = "ctf_id")]
    pub ctf_id: i64,
    pub weight: f64,
    pub duration: DurationDto,
    #[serde(rename = "live_feed")]
    pub live_feed: String,
    pub logo: String,
    pub id: i64,
    pub title: String,
    pub start: String,
    pub participants: i64,
    pub location: String,
    pub finish: String,
    pub description: String,
    pub format: String,
    #[serde(rename = "is_votable_now")]
    pub is_votable_now: bool,
    pub prizes: String,
    #[serde(rename = "format_id")]
    pub format_id: i64,
    pub onsite: bool,
    pub restrictions: String,
    pub url: String,
    #[serde(rename = "public_votable")]
    pub public_votable: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizerDto {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurationDto {
    pub hours: i64,
    pub days: i64,
}
