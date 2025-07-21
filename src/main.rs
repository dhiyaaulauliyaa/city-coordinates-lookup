use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
struct Country {
    id: u32,

    #[serde(rename = "iso2")]
    code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct City {
    id: u32,
    name: String,

    #[serde(default)]
    latitude: Option<String>,

    #[serde(default)]
    longitude: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
struct State {
    id: u32,
    country_id: u32,
    name: String,

    #[serde(default)]
    state_code: Option<String>,

    #[serde(default)]
    latitude: Option<String>,

    #[serde(default)]
    longitude: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cities: Vec<City>,
}

#[derive(Debug, thiserror::Error)]
enum ProcessingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File too large: {size} bytes (max: {max_size})")]
    FileTooLarge { size: u64, max_size: u64 },
}

type Result<T> = std::result::Result<T, ProcessingError>;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB limit

fn validate_file_size(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ProcessingError::FileTooLarge {
            size: metadata.len(),
            max_size: MAX_FILE_SIZE,
        });
    }
    Ok(())
}

fn load_countries(raw_dir: &Path) -> Result<HashMap<u32, String>> {
    let countries_path = raw_dir.join("countries.json");
    validate_file_size(&countries_path)?;

    println!("📊 Loading countries from {countries_path:?}");
    let countries_data = fs::read_to_string(&countries_path)?;
    let countries: Vec<Country> = serde_json::from_str(&countries_data)?;

    let mut country_map = HashMap::new();
    for country in countries {
        country_map.insert(country.id, country.code);
    }

    println!("✅ Loaded {} countries", country_map.len());
    Ok(country_map)
}

fn load_states(raw_dir: &Path) -> Result<Vec<State>> {
    let states_path = raw_dir.join("states+cities.json");
    validate_file_size(&states_path)?;

    println!("📊 Loading states and cities from {states_path:?}");
    let states_data = fs::read_to_string(&states_path)?;
    let states: Vec<State> = serde_json::from_str(&states_data)?;

    println!("✅ Loaded {} states", states.len());
    Ok(states)
}

fn group_states_by_country(states: Vec<State>) -> HashMap<u32, Vec<State>> {
    let mut by_country: HashMap<u32, Vec<State>> = HashMap::new();
    for state in states {
        by_country.entry(state.country_id).or_default().push(state);
    }
    by_country
}

fn write_country_files(
    by_country: HashMap<u32, Vec<State>>,
    country_map: &HashMap<u32, String>,
    out_dir: &Path,
) -> Result<()> {
    let total_countries = by_country.len();
    println!("📝 Writing {total_countries} country files...");

    for (i, (country_id, states)) in by_country.iter().enumerate() {
        let code = country_map
            .get(country_id)
            .map(String::as_str)
            .unwrap_or("XX");
        let filename = format!("{country_id}_{code}.json");
        let out_path = out_dir.join(&filename);

        let json = serde_json::to_string_pretty(states)?;
        fs::write(&out_path, json)?;

        let progress = (i + 1) * 100 / total_countries;
        println!(
            "🔄 [{progress:3}%] Wrote {filename} with {} states",
            states.len()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("🌍 City Coordinates Lookup - Data Processor");
    println!("{}", "=".repeat(50));

    // Setup directories
    let raw_dir = Path::new("data").join("raw");
    let out_dir = Path::new("data").join("generated").join("per-country");

    fs::create_dir_all(&out_dir)?;

    // Load and process data
    let country_map = load_countries(&raw_dir)?;
    let states = load_states(&raw_dir)?;
    let by_country = group_states_by_country(states);

    write_country_files(by_country, &country_map, &out_dir)?;

    println!("✅ Processing complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_country_parsing() {
        let json = r#"[
            {"id": 1, "iso2": "US"},
            {"id": 2, "iso2": "CA"}
        ]"#;

        let countries: Vec<Country> = serde_json::from_str(json).unwrap();
        assert_eq!(countries.len(), 2);
        assert_eq!(countries[0].id, 1);
        assert_eq!(countries[0].code, "US");
        assert_eq!(countries[1].id, 2);
        assert_eq!(countries[1].code, "CA");
    }

    #[test]
    fn test_state_parsing() {
        let json = r#"[
            {
                "id": 1,
                "country_id": 1,
                "name": "California",
                "state_code": "CA",
                "latitude": "36.7783",
                "longitude": "-119.4179",
                "cities": [
                    {
                        "id": 1,
                        "name": "Los Angeles",
                        "latitude": "34.0522",
                        "longitude": "-118.2437"
                    }
                ]
            }
        ]"#;

        let states: Vec<State> = serde_json::from_str(json).unwrap();
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].id, 1);
        assert_eq!(states[0].country_id, 1);
        assert_eq!(states[0].name, "California");
        assert_eq!(states[0].state_code, Some("CA".to_string()));
        assert_eq!(states[0].cities.len(), 1);
        assert_eq!(states[0].cities[0].name, "Los Angeles");
    }

    #[test]
    fn test_group_states_by_country() {
        let states = vec![
            State {
                id: 1,
                country_id: 1,
                name: "California".to_string(),
                state_code: Some("CA".to_string()),
                latitude: None,
                longitude: None,
                cities: vec![],
            },
            State {
                id: 2,
                country_id: 1,
                name: "New York".to_string(),
                state_code: Some("NY".to_string()),
                latitude: None,
                longitude: None,
                cities: vec![],
            },
            State {
                id: 3,
                country_id: 2,
                name: "Ontario".to_string(),
                state_code: Some("ON".to_string()),
                latitude: None,
                longitude: None,
                cities: vec![],
            },
        ];

        let grouped = group_states_by_country(states);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&1).unwrap().len(), 2);
        assert_eq!(grouped.get(&2).unwrap().len(), 1);
    }

    #[test]
    fn test_validate_file_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        // Create a small file
        fs::write(&file_path, "{\"test\": \"data\"}").unwrap();
        assert!(validate_file_size(&file_path).is_ok());

        // Test with non-existent file
        let non_existent = dir.path().join("nonexistent.json");
        assert!(validate_file_size(&non_existent).is_err());
    }

    #[test]
    fn test_format_filename() {
        let filename = format!("{country_id}_{code}.json", country_id = 1, code = "US");
        assert_eq!(filename, "1_US.json");

        let filename = format!("{country_id}_{code}.json", country_id = 999, code = "XX");
        assert_eq!(filename, "999_XX.json");
    }
}
