// Copyright © Aptos Foundation

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use nft_metadata_crawler_utils::publish_to_queue;
use reqwest::Client;
use std::{
    convert::Infallible,
    env,
    fs::File,
    io::{self, BufRead, BufReader},
};

fn process_file() -> Result<Vec<String>, io::Error> {
    let file = File::open("./test.csv")?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

#[tokio::main]
async fn main() {
    let addr = ([0, 0, 0, 0], 8080).into();
    let make_svc = make_service_fn(|_socket: &AddrStream| async move {
        Ok::<_, Infallible>(service_fn(move |req: Request<Body>| async move {
            let client = Client::new();
            let auth = env::var("AUTH").expect("No AUTH");
            let topic_name = env::var("TOPIC_NAME").expect("No TOPIC_NAME");
            let mut parts = req.uri().path().trim_start_matches('/').split('/');
            let start = parts.next();
            let end = parts.next();

            match (start, end) {
                (Some(start), Some(end)) => {
                    if let (Ok(start_num), Ok(end_num)) = (start.parse::<i32>(), end.parse::<i32>())
                    {
                        if let Ok(links) = process_file() {
                            let mut successes = Vec::new();
                            for link in links {
                                match publish_to_queue(
                                    &client,
                                    link.to_string(),
                                    &auth,
                                    &topic_name,
                                )
                                .await
                                {
                                    Ok(link) => successes.push(link),
                                    Err(e) => println!("Error publishing to queue: {}", e),
                                }
                            }
                            Ok::<_, Infallible>(Response::new(Body::from(format!(
                                "Start: {}, End: {}\n{}",
                                start_num,
                                end_num,
                                successes.join("\n")
                            ))))
                        } else {
                            Ok::<_, Infallible>(Response::new(Body::from(format!(
                                "Failed to process file"
                            ))))
                        }
                    } else {
                        Ok::<_, Infallible>(Response::new(Body::from(
                            "Unable to parse start and end transaction_versions",
                        )))
                    }
                },
                _ => Ok::<_, Infallible>(Response::new(Body::from(
                    "Invalid or missing start and end transaction_versions",
                ))),
            }
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
