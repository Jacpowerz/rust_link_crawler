use reqwest;
use scraper::{Html, Selector};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::time::Instant;

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
    let mut found = HashSet::new();
    let mut results = get_links_from_page(url).await?;

    found.extend(results.clone());

    for _ in 0..depth - 1 {
        for url in results.iter() {
            found.extend(get_links_from_page(url).await?);
            searched.insert(url.clone());
        }
        results = found
            .difference(&searched)
            .cloned()
            .collect();
    }
    return Ok(found);
}

#[tokio::main]
async fn main() {
	
	println!("Please enter a link: ");
	let mut user_link = String::new();
	io::stdin().read_line(&mut user_link).expect("Failed to read line");
	
	println!("Please enter a depth: ");
	let mut user_depth = String::new();
	io::stdin().read_line(&mut user_depth).expect("Failed to read line");
	
	let user_depth: i32 = match user_depth.trim().parse() {
             Ok(num) => num,
             Err(_) => panic!("Invalid input")
        };
	println!("Starting Search...");

	let start_time = Instant::now();
	match recursive_get_links(&user_link, user_depth).await {
        Ok(links) => {
            println!("Found links: {:?}", links);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);
    println!("Elapsed time: {} secs", elapsed_time.as_secs());
}
