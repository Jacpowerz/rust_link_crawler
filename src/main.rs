use reqwest;
use scraper::{Html, Selector};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::time::Instant;
use std::env;
use std::error::Error;

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
									matched.as_str()
								)
							);
						}
					}
				}	
			}
			return Ok(links);
		}
		Err(err) => {
			eprintln!("Error: {}", err);
            return Ok(HashSet::new());
		}
	} 
}

async fn recursive_get_links(url: &str, depth: i32) -> Result<HashSet<String>, Box<dyn Error>> {
	
    let mut searched = HashSet::new();
    let mut found = HashSet::new();
    let mut results = get_links_from_page(url).await?;

	println!("{:?}", results);
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
        println!("{:?}", results);
    }
    return Ok(found);
}

#[tokio::main]
async fn main() {
	

	let args: Vec<String> = env::args().collect();
	
	if args.len() <= 2 {
        eprintln!("Please provide command-line arguments.");
        std::process::exit(1);
    }
	
	let url = &args[1];
    let depth = args[2].parse().unwrap();

	eprintln!("Starting Search...");

	let start_time = Instant::now();
	
	let links = recursive_get_links(&url, depth).await;
	
    let end_time = Instant::now();
    
    eprintln!("Number of links found: {}", links.expect("Couldn't get length").len());
    let elapsed_time = end_time.duration_since(start_time);
    eprintln!("Elapsed time: {} secs", elapsed_time.as_secs());
}
