use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Cache for geocoding results to avoid repeated API calls and respect rate limits
lazy_static::lazy_static! {
    static ref GEOCODE_CACHE: Mutex<GeocodeCache> = Mutex::new(GeocodeCache::new());
}

/// Cache for geocoding results with rate limiting
struct GeocodeCache {
    cache: HashMap<String, CachedLocation>,
    last_request: Option<Instant>,
}

#[derive(Clone)]
struct CachedLocation {
    location: String,
    cached_at: Instant,
}

impl GeocodeCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            last_request: None,
        }
    }

    /// Get cached location or None if not cached/expired (1 hour expiry)
    fn get(&self, lat: f64, lon: f64) -> Option<String> {
        let key = format!("{:.4},{:.4}", lat, lon);
        self.cache.get(&key).and_then(|cached| {
            if cached.cached_at.elapsed() < Duration::from_secs(3600) {
                Some(cached.location.clone())
            } else {
                None
            }
        })
    }

    /// Store location in cache
    fn set(&mut self, lat: f64, lon: f64, location: String) {
        let key = format!("{:.4},{:.4}", lat, lon);
        self.cache.insert(
            key,
            CachedLocation {
                location,
                cached_at: Instant::now(),
            },
        );
    }

    /// Check if we need to rate limit (1 request per second for Nominatim)
    fn should_rate_limit(&self) -> bool {
        self.last_request
            .map(|last| last.elapsed() < Duration::from_secs(1))
            .unwrap_or(false)
    }

    /// Update last request time
    fn mark_request(&mut self) {
        self.last_request = Some(Instant::now());
    }
}

/// Nominatim API response structure
#[derive(Debug, Deserialize)]
struct NominatimResponse {
    address: Option<Address>,
}

#[derive(Debug, Deserialize)]
struct Address {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    suburb: Option<String>,
    state: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
}

/// Reverse geocode GPS coordinates to a location name using Nominatim
/// Returns location in format like "Seattle_WA" or "Paris_France"
pub fn reverse_geocode(lat: f64, lon: f64) -> Option<String> {
    // Check cache first
    {
        let cache = GEOCODE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(lat, lon) {
            log::debug!("Geocode cache hit for {:.4},{:.4}", lat, lon);
            return Some(cached);
        }
    }

    // Rate limiting check
    {
        let cache = GEOCODE_CACHE.lock().unwrap();
        if cache.should_rate_limit() {
            log::debug!("Rate limiting geocoding request");
            // Fall back to coordinates format
            return None;
        }
    }

    // Make API request
    match geocode_from_nominatim(lat, lon) {
        Ok(location) => {
            // Cache the result
            let mut cache = GEOCODE_CACHE.lock().unwrap();
            cache.set(lat, lon, location.clone());
            cache.mark_request();
            Some(location)
        }
        Err(e) => {
            log::warn!("Geocoding failed: {}", e);
            None
        }
    }
}

/// Make the actual API request to Nominatim
fn geocode_from_nominatim(lat: f64, lon: f64) -> Result<String> {
    // Build the User-Agent with version and contact email
    let user_agent = format!(
        "Nameback/{} (https://github.com/h4x0r/nameback; info@securityronin.com)",
        env!("CARGO_PKG_VERSION")
    );

    // Build the request URL
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&zoom=10",
        lat, lon
    );

    log::debug!("Geocoding {},{} via Nominatim", lat, lon);

    // Use blocking reqwest since we're in a sync context
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(&url)
        .send()
        .context("Failed to send geocoding request")?;

    if !response.status().is_success() {
        anyhow::bail!("Geocoding API returned status: {}", response.status());
    }

    let data: NominatimResponse = response
        .json()
        .context("Failed to parse geocoding response")?;

    // Extract location from response
    if let Some(address) = data.address {
        let location = format_location_from_address(&address);
        if !location.is_empty() {
            return Ok(location);
        }
    }

    anyhow::bail!("No location data in geocoding response")
}

/// Format the address into a filename-friendly location string
fn format_location_from_address(address: &Address) -> String {
    // Get city/town/village name
    let city = address
        .city
        .as_ref()
        .or(address.town.as_ref())
        .or(address.village.as_ref())
        .or(address.hamlet.as_ref())
        .or(address.suburb.as_ref());

    // Get state/country
    let region = if let Some(country_code) = &address.country_code {
        if country_code == "us" || country_code == "ca" {
            // For US/Canada, use state abbreviation if available
            address.state.as_ref()
        } else {
            // For other countries, use country name
            address.country.as_ref()
        }
    } else {
        address.country.as_ref()
    };

    // Combine city and region
    match (city, region) {
        (Some(c), Some(r)) => {
            // Clean and format for filename
            let city_clean = clean_for_filename(c);
            let region_clean = clean_for_filename(r);

            // For US states, try to abbreviate
            let region_abbrev = if address.country_code.as_deref() == Some("us") {
                abbreviate_us_state(&region_clean).unwrap_or(region_clean)
            } else {
                region_clean
            };

            format!("{}_{}", city_clean, region_abbrev)
        }
        (Some(c), None) => clean_for_filename(c),
        (None, Some(r)) => clean_for_filename(r),
        (None, None) => String::new(),
    }
}

/// Clean a string for use in filename (remove special chars, spaces to underscores)
fn clean_for_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '_'
            } else {
                '_'
            }
        })
        .filter(|&c| c != '_' || c.is_alphanumeric())
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Try to abbreviate US state names
fn abbreviate_us_state(state: &str) -> Option<String> {
    let state_lower = state.to_lowercase();
    let abbrev = match state_lower.as_str() {
        "alabama" => "AL",
        "alaska" => "AK",
        "arizona" => "AZ",
        "arkansas" => "AR",
        "california" => "CA",
        "colorado" => "CO",
        "connecticut" => "CT",
        "delaware" => "DE",
        "florida" => "FL",
        "georgia" => "GA",
        "hawaii" => "HI",
        "idaho" => "ID",
        "illinois" => "IL",
        "indiana" => "IN",
        "iowa" => "IA",
        "kansas" => "KS",
        "kentucky" => "KY",
        "louisiana" => "LA",
        "maine" => "ME",
        "maryland" => "MD",
        "massachusetts" => "MA",
        "michigan" => "MI",
        "minnesota" => "MN",
        "mississippi" => "MS",
        "missouri" => "MO",
        "montana" => "MT",
        "nebraska" => "NE",
        "nevada" => "NV",
        "new hampshire" => "NH",
        "new jersey" => "NJ",
        "new mexico" => "NM",
        "new york" => "NY",
        "north carolina" => "NC",
        "north dakota" => "ND",
        "ohio" => "OH",
        "oklahoma" => "OK",
        "oregon" => "OR",
        "pennsylvania" => "PA",
        "rhode island" => "RI",
        "south carolina" => "SC",
        "south dakota" => "SD",
        "tennessee" => "TN",
        "texas" => "TX",
        "utah" => "UT",
        "vermont" => "VT",
        "virginia" => "VA",
        "washington" => "WA",
        "west virginia" => "WV",
        "wisconsin" => "WI",
        "wyoming" => "WY",
        _ => return None,
    };
    Some(abbrev.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_for_filename() {
        assert_eq!(clean_for_filename("New York"), "New_York");
        assert_eq!(clean_for_filename("Saint-Denis"), "Saint_Denis");
        assert_eq!(clean_for_filename("Los Angeles, CA"), "Los_Angeles_CA");
    }

    #[test]
    fn test_abbreviate_us_state() {
        assert_eq!(abbreviate_us_state("California"), Some("CA".to_string()));
        assert_eq!(abbreviate_us_state("New York"), Some("NY".to_string()));
        assert_eq!(abbreviate_us_state("washington"), Some("WA".to_string()));
        assert_eq!(abbreviate_us_state("Ontario"), None);
    }
}