use clap::Parser;

#[derive(Parser)]
#[command(name = "butterfly-scraper")]
#[command(about = "A CLI tool for scraping butterfly and moth sighting data")]
#[command(version = "1.0")]
pub(crate) struct Args {
    /// Minimum sighting ID to scrape
    #[arg(short, long)]
    pub min: u64,

    /// Maximum sighting ID to scrape
    #[arg(short = 'M', long)]
    pub max: u64,

    /// Base delay between requests in milliseconds
    #[arg(short, long, default_value = "500")]
    pub delay: u64,

    /// Maximum number of concurrent requests
    #[arg(short, long, default_value = "5")]
    pub concurrent: usize,
    
    #[arg(short, long, default_value = "missing.txt")]
    pub missing: String,

    /// Maximum number of retry attempts
    #[arg(short, long, default_value = "3")]
    pub retries: u32,

    /// Output CSV filename
    #[arg(short, long, default_value = "sightings.csv")]
    pub output: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
