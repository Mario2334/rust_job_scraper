mod models;

use std::ops::Deref;
use reqwest::header::{HeaderMap, USER_AGENT};
use scraper::{Html, Selector};
use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use chrono::DateTime;
use crate::models::{Job, JobFactory};

async fn make_job_request() -> Vec<Job> {
    // Extract some useful information from the request
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36".parse().unwrap());

    // Make the request to the Upwork job feed
    let request = reqwest::Client::new()
        .get("https://www.upwork.com/ab/feed/topics/rss?securityToken=1a5ce4b31dc1f7c28bd88ecdc53686c076ae996ebc26f544d38673dc15fb40d1f388bd7de5819d1bb7fe68e19653e8e32af7d607728cc69ca9379d4f08697baf&userUid=868390199689519104&orgUid=868390199693713409&topic=6679312")
        .headers(headers)
        .send().await.unwrap();
    let body = request.text().await.unwrap();
    // Parse the HTML with the scraper library
    let document = Html::parse_document(&body);
    let selector = Selector::parse("item").unwrap();

    let mut job_store: Vec<Job> = vec![];

    // Iterate over the job postings and print out the title and link
    for job in document.select(&selector) {
        let title = job.select(&Selector::parse("title").unwrap()).next().unwrap().text().collect::<String>();
        let link = job.select(&Selector::parse("guid").unwrap()).next().unwrap().text().collect::<String>();
        let description = job.select(&Selector::parse("description").unwrap()).next().unwrap().text().collect::<String>();
        let date = job.select(&Selector::parse("pubDate").unwrap()).next().unwrap().text().collect::<String>();
        let dt = DateTime::parse_from_rfc2822(date.as_str()).unwrap();

        println!("{}", link);

        let job_struct = Job::new(title, link, description, dt);

        job_store.insert(job_store.len(),job_struct);
        // println!("{} - {}", title, link);
    }
    return job_store;
}


/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    let job_store = make_job_request().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await;

    Ok(())
}
