use serde::{Deserialize, Serialize};
use chrono::{DateTime, FixedOffset};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Job {
    #[serde(default, skip_serializing)]
    id: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "URL")]
    link: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Date Added")]
    date: DateTime<FixedOffset>
}

pub trait JobFactory {
    fn new(title: String, link: String, description: String, date: DateTime<FixedOffset>, source: String) -> Self;
}

impl JobFactory for Job {
    fn new(title: String, link: String, description: String, date: DateTime<FixedOffset>, source: String) -> Self {
        Job {
            title,
            link,
            description,
            source,
            date,
            .. Default::default()
        }
    }
}

impl airtable::Record for Job {
    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn id(&self) -> &str {
        &self.id
    }
}
