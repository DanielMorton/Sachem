use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SightingRecord {
    pub sighting_id: Option<u64>,
    pub url: Option<String>,
    pub common_name: String,
    pub scientific_name: String,
    pub species_link: String,
    pub observation_date: String,
    pub submitted_by: String,
    pub specimen_type: String,
    pub status: String,
    pub verified_by: String,
    pub verified_date: String,
    pub checklist_regions: String,
}

impl Default for SightingRecord {
    fn default() -> Self {
        Self {
            sighting_id: None,
            url: None,
            common_name: String::new(),
            scientific_name: String::new(),
            species_link: String::new(),
            observation_date: String::new(),
            submitted_by: String::new(),
            specimen_type: String::new(),
            status: String::new(),
            verified_by: String::new(),
            verified_date: String::new(),
            checklist_regions: String::new(),
        }
    }
}
