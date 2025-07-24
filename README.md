# Sachem: Butterfly & Moth Sighting Scraper

A high-performance, concurrent web scraper for collecting butterfly and moth sighting data from butterfliesandmoths.org. Built with Rust for speed, reliability, and efficient data collection.

## Features

- **Concurrent Scraping**: Process multiple sighting pages simultaneously with configurable concurrency limits
- **Intelligent Retry Logic**: Exponential backoff with jitter for handling rate limits and network issues
- **Missing Data Tracking**: Automatically tracks and persists failed sighting IDs for later retry
- **Progress Monitoring**: Real-time progress bars with time estimates
- **CSV Export**: Clean, structured data output in CSV format
- **Robust Error Handling**: Graceful handling of network errors, rate limits, and malformed data
- **Configurable Delays**: Respectful scraping with customizable request delays

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

### Build from Source

```bash
git clone <repository-url>
cd butterfly-scraper
cargo build --release
```

The compiled binary will be available at `target/release/butterfly-scraper`.

## Usage

### Basic Usage

Scrape sightings from ID 1000000 to 1002000:

```bash
./butterfly-scraper --min 1000000 --max 1002000
```

### Advanced Options

```bash
./butterfly-scraper \
    --min 1000 \
    --max 5000 \
    --delay 1000 \
    --concurrent 10 \
    --retries 5 \
    --output sightings.csv \
    --missing failed_ids.txt
```

### Command Line Arguments

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--min` | `-m` | 0 | Minimum sighting ID to scrape |
| `--max` | `-M` | *required* | Maximum sighting ID to scrape |
| `--delay` | `-d` | 500 | Base delay between requests (milliseconds) |
| `--concurrent` | `-c` | 5 | Maximum concurrent requests |
| `--retries` | `-r` | 3 | Maximum retry attempts per request |
| `--output` | `-o` | sightings.csv | Output CSV filename |
| `--missing` | | missing.txt | File to track failed sighting IDs |
| `--verbose` | `-v` | false | Enable verbose logging |

## Data Structure

Each scraped sighting record contains the following fields:

```csv
sighting_id,url,common_name,scientific_name,species_link,observation_date,submitted_by,specimen_type,observation_notes,status,verified_by,verified_date,coordinator_notes,checklist_regions
```

### Field Descriptions

- **sighting_id**: Unique numerical identifier for the sighting
- **url**: URL of the sighting page
- **common_name**: Common name of the species (e.g. "Monarch Butterfly")
- **scientific_name**: Scientific name of the species (e.g. "Danaus plexippus")
- **species_link**: Relative link to species information page
- **observation_date**: Date when the sighting was observed
- **submitted_by**: Username of the person who submitted the sighting
- **specimen_type**: Type of specimen (e.g., "Live adult", "Photograph")
- **observation_notes**: Notes from the observer
- **status**: Verification status (e.g., "Verified", "Pending")
- **verified_by**: Username of the verifier (if verified)
- **verified_date**: Date of verification
- **coordinator_notes**: Notes from regional coordinators
- **checklist_regions**: Geographic regions associated with the sighting

## Performance & Best Practices

### Recommended Settings

For respectful scraping that balances speed with server load:

```bash
# Light usage (recommended for initial testing)
--delay 1000 --concurrent 3 --retries 3

# Moderate usage (good balance of speed and politeness)
--delay 500 --concurrent 5 --retries 3

# Heavy usage (use sparingly, monitor for rate limiting)
--delay 200 --concurrent 10 --retries 5
```

### Rate Limiting

The scraper includes several mechanisms to handle rate limiting:

1. **Base delays**: Configurable delay between requests
2. **Jitter**: Random variation in delays to avoid thundering herd
3. **Exponential backoff**: Increasing delays for retries
4. **429 handling**: Automatic retry on rate limit responses

### Missing Sightings Recovery

Failed sighting IDs are automatically tracked in a file. To retry only the failed sightings:

```bash
# After initial run, check missing.txt for failed IDs
cat missing.txt

# Create a custom script to retry specific ranges or use the missing file
# for targeted re-scraping
```

## Output Examples

### Hypothetical Console Output

```
Scraping multiple sightings...
[00:02:34] ████████████████████████████████████████ 1000/1000 100% ETA: 00:00:00 Scraping sightings
Elapsed time: 00:02:34.567
Successfully scraped 987 out of 1000 sightings (13 missing)
Data saved to sightings.csv

Summary:
Total sightings: 987
Unique species: 234
Date range: 2020-03-15 to 2024-01-20
Top regions:
  Ontario, Canada: 156
  New York, United States: 134
  Pennsylvania, United States: 98
```

### CSV Output Sample

```csv
sighting_id,url,common_name,scientific_name,species_link,observation_date,submitted_by,specimen_type,observation_notes,status,verified_by,verified_date,coordinator_notes,checklist_regions
123456,https://www.butterfliesandmoths.org/sighting_details/123456,Monarch,Danaus plexippus,/species/Danaus-plexippus,2024-01-15,observer123,Live adult,Observed during migration,Verified,coordinator456,2024-01-16,,Ontario Canada
```

## Error Handling

The scraper handles various error conditions gracefully:

- **Network timeouts**: Automatic retry with exponential backoff
- **Rate limiting (429)**: Intelligent delay and retry
- **Missing pages (404)**: Logged and tracked in missing sightings file
- **Malformed HTML**: Skipped with warning, ID added to missing list
- **Connection errors**: Retry with increasing delays

## Logging

Set the `RUST_LOG` environment variable for detailed logging:

```bash
# Error level only
RUST_LOG=error ./butterfly-scraper --min 1000 --max 2000

# Info level (recommended)
RUST_LOG=info ./butterfly-scraper --min 1000 --max 2000

# Debug level (verbose)
RUST_LOG=debug ./butterfly-scraper --min 1000 --max 2000
```

## Dependencies

- **clap 4.5.41**: Command-line argument parsing with derive macros
- **csv 1.3.1**: Efficient CSV reading and writing
- **env_logger 0.10.2**: Environment-based logging configuration
- **futures 0.3.31**: Async utilities and combinators
- **indicatif 0.18.0**: Progress bars and status indicators
- **log 0.4.27**: Logging facade for structured output
- **rand 0.9.2**: Random number generation for jitter and delays
- **reqwest 0.12.22**: HTTP client with JSON support and async capabilities
- **scraper 0.23.1**: HTML parsing and CSS selector support
- **serde 1.0.219**: Serialization/deserialization with derive macros
- **tokio 1.46.1**: Full-featured async runtime for concurrent operations

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_scraper_creation
```

## License

[Add your license information here]

## Support

For issues, questions, or contributions, please [create an issue](link-to-issues) in the repository.

---

**Disclaimer**: This tool is for educational and research purposes. Users are responsible for ensuring compliance with applicable terms of service and legal requirements.
