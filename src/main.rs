use reqwest;
use scraper::{Html, Selector};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;
use std::env;
use tokio::task::spawn_blocking;

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
    static ref PATTERN: Regex = Regex::new(r"^(https?://[^/]+(?:/\S*)?)").unwrap();
    static ref SELECTOR: Selector = Selector::parse("a").unwrap();
}

async fn get_links_from_page(url: &str) -> Result<HashSet<String>, reqwest::Error> {
	
	let mut links = HashSet::new();

    match CLIENT.get(url).send().await {
	
		Ok(response) => {
			let html = response.text().await?;
		
			let document = Html::parse_document(&html);
			let selector = &SELECTOR;
			
			for link in document.select(&selector) {
				if let Some(link) = link.value().attr("href") {
					if let Some(captures) = PATTERN.captures(link) {
						if let Some(matched) = captures.get(1) {
							links.insert(
								String::from(
									matched
										.as_str()
								)
							);
						}
					}
				}	
			}
			return Ok(links);
		}
		Err(err) => {
			eprintln!("Error processing {}: {}", url, err);
            return Ok(HashSet::new());
		}
	} 
}

async fn recursive_get_links(url: &str, depth: i32) -> Result<HashSet<String>, Box<dyn Error>> {
	
    let mut searched = HashSet::new();
    let mut found = get_links_from_page(url).await?;

    for _ in 0..depth - 1 {
		let mut tasks = Vec::new();
		
		found.difference(&searched).for_each(|url| {
			tasks.push(spawn_blocking(|| get_links_from_page(url)));
            searched.insert(url.clone());
		});
        
        for task in tasks {
			match task.await {
				Ok(result) => found.extend(result.await.unwrap_or_default()),
				Err(err) => eprintln!("Error appending task: {}", err),
			}
        }
    }
    return Ok(found)
}

#[tokio::main]
async fn main() {
	
	let args: Vec<String> = env::args().collect();

    let url = &args[1];
    let depth = args[2].parse().unwrap();

	println!("Starting Search...");

	let start_time = Instant::now();
	
	match recursive_get_links(&url, depth).await {
        Ok(links) => {
            println!("Found links: {:?}\nNumber of links: {}", links, links.len());
        }
        Err(err) => {
            eprintln!("Error. Search ended prematurely: {}", err);
        }
    }
    let end_time = Instant::now();
    
    let elapsed_time = end_time.duration_since(start_time);
    println!("Elapsed time: {} secs", elapsed_time.as_secs());
}
