mod models;

use std::collections::HashSet;
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
        let mut title = job.select(&Selector::parse("title").unwrap()).next().unwrap().text().collect::<String>();
        let link = job.select(&Selector::parse("guid").unwrap()).next().unwrap().text().collect::<String>();
        let description = job.select(&Selector::parse("description").unwrap()).next().unwrap().text().collect::<String>();
        let date = job.select(&Selector::parse("pubDate").unwrap()).next().unwrap().text().collect::<String>();
        let dt = DateTime::parse_from_rfc2822(date.as_str()).unwrap();
        title = title.replace("<![CDATA[", "");
        title = title.replace("]]>", "");

        println!("{}", link);

        let job_struct = Job::new(title, link, description,dt, String::from("Upwork"));

        job_store.insert(job_store.len(),job_struct);
        // println!("{} - {}", title, link);
    }
    return job_store;
}

fn get_job_airtable() -> Vec<Job> {
    let base = airtable::new::<Job>("keyBjjQARXZg2S0Rw", "appcFeE1g7vQFPGVB", "tblFRvg1Xplqf7SW5");
    let results = base.query().into_iter().collect();
    results
}

async fn push_table(job_list: Vec<Job>) {
    let base = airtable::new::<Job>("keyBjjQARXZg2S0Rw", "appcFeE1g7vQFPGVB", "tblFRvg1Xplqf7SW5");
    for job in job_list {
        let created_job = base.create(&job);
        match created_job {
            Err(e) => println!("{}", e),
            _ => {}
        }
        // println!("{:?}", created_job);
    }

    // let mut records: Vec<Record<Job>> = vec![];
    // for job in job_list {
    //     let record = Record{
    //         id: "".to_string(),
    //         fields: job,
    //         created_time: None,
    //     };
    //     records.insert(records.len(),record);
    // }
    //
    // let airtable = Airtable::new("keyBjjQARXZg2S0Rw", "appcFeE1g7vQFPGVB","");
    //
    // airtable.create_records("tblFRvg1Xplqf7SW5", records).await.expect("TODO: panic message");
}

async fn compare_with_current_data(scrapped_job_list: HashSet<Job>) -> Vec<Job> {
    let results: HashSet<Job> = get_job_airtable().into_iter().collect();
    let compare: Vec<Job> = (&scrapped_job_list - &results).iter().cloned().collect();
    compare
}


/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    let mut job_store = make_job_request().await;
    // for job in &job_store {
    //     println!("{}", job);
    // }
    job_store = compare_with_current_data(job_store.into_iter().collect()).await;
    // for job in &job_store {
    //     println!("{}", job);
    // }
    push_table(job_store).await;
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
