use serde::{Deserialize, Serialize};
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Deserialize, Serialize)]
pub struct Job {
    title: String,
    link: String,
    description: String,
    date: DateTime<FixedOffset>


}

pub trait JobFactory {
    fn new(title: String, link: String, description: String, date: DateTime<FixedOffset>) -> Self;
}

impl JobFactory for Job {
    fn new(title: String, link: String, description: String, date: DateTime<FixedOffset>) -> Self {
        Job {
            title,
            link,
            description,
            date,
        }
    }
}