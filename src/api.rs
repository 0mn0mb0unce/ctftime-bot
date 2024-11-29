pub mod ctftime_api {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use reqwest::StatusCode;

    use crate::EventDto;

    pub async fn get_events(
        start_time: SystemTime,
        finish_time: SystemTime,
    ) -> Result<Vec<EventDto>, Box<dyn std::error::Error>> {
        let start_time_ts = start_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let finish_time_ts = finish_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let url = format!(
            "https://ctftime.org/api/v1/events/?limit=200&start={}&finish={}",
            start_time_ts, finish_time_ts
        );
        let resp = reqwest::get(url).await?.json::<Vec<EventDto>>().await?;
        Ok(resp)
    }

    pub async fn get_event_by_id(id: u64) -> Result<Option<EventDto>, Box<dyn std::error::Error>> {
        let url = format!("https://ctftime.org/api/v1/events/{}/", id);
        let resp = reqwest::get(url).await?;
        if resp.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Ok(Some(resp.json::<EventDto>().await?))
        }
    }
}
