pub mod ctftime_api {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use crate::EventDto;

    pub async fn get_events() -> Result<Vec<EventDto>, Box<dyn std::error::Error>> {
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

    pub async fn get_event_by_id(id: u64) -> Result<EventDto, Box<dyn std::error::Error>> {
        let url = format!("https://ctftime.org/api/v1/events/{}/", id);
        let resp = reqwest::get(url).await?.json::<EventDto>().await?;

        Ok(resp)
    }
}
