use crate::record::SightingRecord;
use csv::Writer;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use rand::Rng;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub struct ButterflyMothScraper {
    client: Client,
    pub(crate) base_delay: Duration,
    pub(crate) max_retries: u32,
    pub missing_sightings: Arc<Mutex<Vec<u64>>>,
    pub missing_sightings_file: Option<String>,
}

impl ButterflyMothScraper {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_delay: Duration::from_millis(1000),
            max_retries: 3,
            missing_sightings: Arc::new(Mutex::new(Vec::new())),
            missing_sightings_file: None,
        }
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.base_delay = Duration::from_millis(delay_ms);
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn with_missing_sightings_file(mut self, filename: &str) -> Self {
        self.missing_sightings_file = Some(filename.to_string());
        // Load existing missing sightings from file
        if let Err(e) = self.load_missing_sightings() {
            warn!("Could not load missing sightings from {}: {}", filename, e);
        }
        self
    }

    /// Load missing sightings from file
    fn load_missing_sightings(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filename) = &self.missing_sightings_file {
            if let Ok(file) = File::open(filename) {
                let reader = BufReader::new(file);
                let mut missing_list = self.missing_sightings.lock().unwrap();

                for line in reader.lines() {
                    let line = line?;
                    if let Ok(sighting_id) = line.trim().parse::<u64>() {
                        missing_list.push(sighting_id);
                    }
                }

                info!("Loaded {} missing sightings from {}", missing_list.len(), filename);
            }
        }
        Ok(())
    }

    /// Get a copy of the missing sightings list
    pub fn get_missing_sightings(&self) -> Vec<u64> {
        self.missing_sightings.lock().unwrap().clone()
    }

    /// Clear the missing sightings list
    pub fn clear_missing_sightings(&self) {
        self.missing_sightings.lock().unwrap().clear();
    }

    /// Add a sighting ID to the missing list
    fn add_missing_sighting(&self, sighting_id: u64) {
        let mut missing_list = self.missing_sightings.lock().unwrap();
        if !missing_list.contains(&sighting_id) {
            missing_list.push(sighting_id);

            // Immediately append to file if configured
            if let Some(filename) = &self.missing_sightings_file {
                if let Err(e) = self.append_missing_sighting_to_file(sighting_id, filename) {
                    error!("Failed to append missing sighting {} to file: {}", sighting_id, e);
                }
            }
        }
    }

    /// Append a single missing sighting ID to the file
    fn append_missing_sighting_to_file(&self, sighting_id: u64, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;

        writeln!(file, "{}", sighting_id)?;
        file.flush()?;

        Ok(())
    }

    /// Filter out missing sightings from a list of sighting IDs
    fn filter_missing_sightings(&self, sighting_ids: &[u64]) -> Vec<u64> {
        let missing_set: HashSet<u64> = self.missing_sightings.lock().unwrap().iter().cloned().collect();
        let filtered: Vec<u64> = sighting_ids.iter()
            .filter(|&id| !missing_set.contains(id))
            .cloned()
            .collect();

        let filtered_count = sighting_ids.len() - filtered.len();
        if filtered_count > 0 {
            info!("Filtered out {} already missing sightings", filtered_count);
        }

        filtered
    }

    /// Parse HTML content into a SightingRecord
    fn parse_html_to_record(&self, html_content: &str) -> Option<SightingRecord> {
        let document = Html::parse_document(html_content);

        // Find rows with views-row class
        let row_selector = Selector::parse("div[class*='views-row']").ok()?;
        let row = document.select(&row_selector).next()?;

        let mut record = SightingRecord::default();

        // Helper function to extract field content
        let get_field = |class_name: &str, sub_selector: &str| -> Option<String> {
            let field_selector = Selector::parse(&format!("div.{}", class_name)).ok()?;
            let field = row.select(&field_selector).next()?;

            let content_selector = Selector::parse(sub_selector).ok()?;
            let content = field.select(&content_selector).next()?;

            Some(
                content
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string(),
            )
        };

        // Extract species information
        if let Ok(species_selector) = Selector::parse("div.views-field-field-sciname") {
            if let Some(species_field) = row.select(&species_selector).next() {
                if let Ok(h4_selector) = Selector::parse("h4") {
                    if let Some(h4) = species_field.select(&h4_selector).next() {
                        // Get common name (text before any child elements)
                        let text_nodes: Vec<_> = h4.text().collect();
                        if !text_nodes.is_empty() {
                            record.common_name = text_nodes[0].trim().to_string();
                        }

                        // Get scientific name from <em> tag
                        if let Ok(em_selector) = Selector::parse("em") {
                            if let Some(em) = h4.select(&em_selector).next() {
                                record.scientific_name =
                                    em.text().collect::<Vec<_>>().join("").trim().to_string();
                            }
                        }

                        // Get species link from <a> tag
                        if let Ok(a_selector) = Selector::parse("a") {
                            if let Some(a) = h4.select(&a_selector).next() {
                                record.species_link =
                                    a.value().attr("href").unwrap_or("").to_string();
                            }
                        }
                    }
                }
            }
        }

        // Extract other fields
        record.observation_date =
            get_field("views-field-field-sightingdate", ".field-content").unwrap_or_default();
        record.submitted_by = get_field("views-field-name", ".username").unwrap_or_default();
        record.specimen_type =
            get_field("views-field-field-specimen-type", ".field-content").unwrap_or_default();
        record.status =
            get_field("views-field-field-sighting-status", ".field-content").unwrap_or_default();
        record.verified_by = get_field("views-field-name-1", ".username").unwrap_or_default();
        record.verified_date =
            get_field("views-field-field-recorddate", ".field-content").unwrap_or_default();
        

        // Extract regions (join multiple links)
        if let Ok(region_selector) = Selector::parse("div.views-field-field-region") {
            if let Some(region_field) = row.select(&region_selector).next() {
                if let Ok(a_selector) = Selector::parse("a") {
                    let regions: Vec<String> = region_field
                        .select(&a_selector)
                        .map(|link| link.text().collect::<Vec<_>>().join("").trim().to_string())
                        .collect();
                    record.checklist_regions = regions.join(", ");
                }
            }
        }

        Some(record)
    }

    /// Scrape a single sighting page by ID with exponential backoff retry
    pub async fn scrape_sighting_page(&self, sighting_id: u64) -> Option<SightingRecord> {
        let url = format!(
            "https://www.butterfliesandmoths.org/sighting_details/{}",
            sighting_id
        );

        for attempt in 0..=self.max_retries {
            // Add delay with jitter
            if attempt > 0 {
                let backoff_delay = Duration::from_millis(
                    (2_u64.pow(attempt) * self.base_delay.as_millis() as u64)
                        + rand::rng().random_range(0..self.base_delay.as_millis() as u64),
                );
                info!(
                    "Retrying sighting {} (attempt {}) after {}ms delay",
                    sighting_id,
                    attempt + 1,
                    backoff_delay.as_millis()
                );
                sleep(backoff_delay).await;
            } else {
                let initial_delay = Duration::from_millis(
                    self.base_delay.as_millis() as u64
                        + rand::rng().random_range(0..self.base_delay.as_millis() as u64 / 2),
                );
                sleep(initial_delay).await;
            }

            match self.client.get(&url).send().await {
                Ok(response) => match response.status().as_u16() {
                    429 => {
                        if attempt < self.max_retries {
                            warn!("Rate limited for sighting {}, retrying...", sighting_id);
                            continue;
                        } else {
                            error!(
                                "Rate limited for sighting {}, max retries reached",
                                sighting_id
                            );
                            self.add_missing_sighting(sighting_id);
                            return None;
                        }
                    }
                    200..=299 => match response.text().await {
                        Ok(html) => match self.parse_html_to_record(&html) {
                            Some(mut record) => {
                                record.sighting_id = Some(sighting_id);
                                record.url = Some(url);
                                if attempt > 0 {
                                    info!(
                                        "Successfully scraped sighting {} on attempt {}",
                                        sighting_id,
                                        attempt + 1
                                    );
                                } else {
                                    info!("Successfully scraped sighting {}", sighting_id);
                                }
                                return Some(record);
                            }
                            None => {
                                warn!("No data found for sighting {}", sighting_id);
                                self.add_missing_sighting(sighting_id);
                                return None;
                            }
                        },
                        Err(_) => {
                            self.add_missing_sighting(sighting_id);
                            return None;
                        }
                    },
                    _ => {
                        if attempt < self.max_retries {
                            warn!(
                                "HTTP error {} for sighting {}, retrying...",
                                response.status(),
                                sighting_id
                            );
                            continue;
                        } else {
                            self.add_missing_sighting(sighting_id);
                            return None;
                        }
                    }
                },
                Err(e) => {
                    if attempt < self.max_retries {
                        warn!(
                            "Request failed for sighting {}, retrying...: {}",
                            sighting_id, e
                        );
                        continue;
                    } else {
                        error!(
                            "Request failed for sighting {}, max retries reached: {}",
                            sighting_id, e
                        );
                        self.add_missing_sighting(sighting_id);
                        return None;
                    }
                }
            }
        }

        error!(
            "Failed to scrape sighting {} after {} attempts",
            sighting_id,
            self.max_retries + 1
        );
        self.add_missing_sighting(sighting_id);
        None
    }

    /// Scrape multiple sighting pages concurrently
    pub async fn scrape_multiple_sightings(
        &self,
        sighting_ids: &[u64],
        max_concurrent: usize,
    ) -> Vec<SightingRecord> {
        let filtered_sightings_ids = self.filter_missing_sightings(sighting_ids);

        // Create progress bar
        let progress_bar = ProgressBar::new(filtered_sightings_ids.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {percent:>3}% ETA: {eta_precise} {msg}")
                .unwrap()
                .progress_chars("##-")
        );
        progress_bar.set_message("Scraping sightings");

        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut tasks = Vec::new();
        let pb = Arc::new(progress_bar);

        for sighting_id in filtered_sightings_ids.iter() {
            let permit = Arc::clone(&semaphore);
            let sighting_id = *sighting_id;
            let scraper = self;
            let progress = Arc::clone(&pb);

            let task = async move {
                let _permit = permit.acquire().await.unwrap();
                let result = scraper.scrape_sighting_page(sighting_id).await;
                progress.inc(1);
                result
            };

            tasks.push(task);
        }

        let results = join_all(tasks).await;
        let successful_records: Vec<SightingRecord> =
            results.into_iter().filter_map(|r| r).collect();

        let missing_count = self.get_missing_sightings().len();
        info!(
            "Successfully scraped {} out of {} sightings ({} unfound)",
            successful_records.len(),
            sighting_ids.len(),
            missing_count
        );

        successful_records
    }

    /// Scrape a range of sighting IDs
    pub async fn scrape_sighting_range(
        &self,
        start_id: u64,
        end_id: u64,
        max_concurrent: usize,
    ) -> Vec<SightingRecord> {
        let sighting_ids: Vec<u64> = (start_id..=end_id).collect();
        self.scrape_multiple_sightings(&sighting_ids, max_concurrent)
            .await
    }

    /// Save records to CSV file
    pub fn save_to_csv(
        &self,
        records: &[SightingRecord],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(filename)?;
        let mut writer = Writer::from_writer(file);

        for record in records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        info!("Data saved to {}", filename);
        Ok(())
    }
}