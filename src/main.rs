mod parse;
mod record;
mod scraper;
mod util;

use crate::parse::Args;
use crate::record::SightingRecord;
use crate::scraper::ButterflyMothScraper;
use crate::util::print_hms;
use clap::Parser;
use std::collections::HashMap;
use std::time::Instant;

/// Utility functions
pub fn get_failed_ids(original_ids: &[u64], scraped_records: &[SightingRecord]) -> Vec<u64> {
    let scraped_ids: std::collections::HashSet<u64> = scraped_records
        .iter()
        .filter_map(|r| r.sighting_id)
        .collect();

    original_ids
        .iter()
        .filter(|id| !scraped_ids.contains(id))
        .copied()
        .collect()
}

pub fn print_summary(records: &[SightingRecord]) {
    if records.is_empty() {
        println!("No records to summarize");
        return;
    }

    println!("\nSummary:");
    println!("Total sightings: {}", records.len());

    let unique_species: std::collections::HashSet<&String> = records
        .iter()
        .map(|r| &r.scientific_name)
        .filter(|name| !name.is_empty())
        .collect();
    println!("Unique species: {}", unique_species.len());

    // Get date range
    let dates: Vec<&String> = records
        .iter()
        .map(|r| &r.observation_date)
        .filter(|date| !date.is_empty())
        .collect();

    if !dates.is_empty() {
        let min_date = dates.iter().min().unwrap();
        let max_date = dates.iter().max().unwrap();
        println!("Date range: {} to {}", min_date, max_date);
    }

    // Top regions
    let mut region_counts: HashMap<&String, usize> = HashMap::new();
    for record in records {
        if !record.checklist_regions.is_empty() {
            *region_counts.entry(&record.checklist_regions).or_insert(0) += 1;
        }
    }

    let mut sorted_regions: Vec<_> = region_counts.into_iter().collect();
    sorted_regions.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Top regions:");
    for (region, count) in sorted_regions.iter().take(3) {
        println!("  {}: {}", region, count);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::try_parse()?;
    // Initialize logger
    env_logger::init();

    let scraper = ButterflyMothScraper::new()
        .with_delay(args.delay) // 500ms base delay
        .with_max_retries(args.retries)
        .with_missing_sightings_file(&args.missing);

    // Example 2: Scrape multiple specific sightings
    println!("\nScraping multiple sightings...");
    let start = Instant::now();
    let records = scraper
        .scrape_sighting_range(args.min, args.max, args.concurrent)
        .await;

    print_hms(&start);
    // Save to CSV
    scraper.save_to_csv(&records, &args.output)?;

    // Print summary
    print_summary(&records);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_scraper_creation() {
        let scraper = ButterflyMothScraper::new();
        assert_eq!(scraper.base_delay, Duration::from_millis(1000));
        assert_eq!(scraper.max_retries, 3);
    }

    #[tokio::test]
    async fn test_scraper_configuration() {
        let scraper = ButterflyMothScraper::new()
            .with_delay(2000)
            .with_max_retries(5);

        assert_eq!(scraper.base_delay, Duration::from_millis(2000));
        assert_eq!(scraper.max_retries, 5);
    }

    #[test]
    fn test_get_failed_ids() {
        let original_ids = vec![1, 2, 3, 4, 5];
        let mut record1 = SightingRecord::default();
        record1.sighting_id = Some(1);
        let mut record3 = SightingRecord::default();
        record3.sighting_id = Some(3);

        let scraped_records = vec![record1, record3];
        let failed_ids = get_failed_ids(&original_ids, &scraped_records);

        assert_eq!(failed_ids, vec![2, 4, 5]);
    }
}
