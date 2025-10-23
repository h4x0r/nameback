use chrono::{NaiveDateTime, Timelike};

/// Represents GPS location data
#[derive(Debug, Clone)]
pub struct LocationData {
    pub latitude: f64,
    pub longitude: f64,
}

/// Extracts GPS coordinates from EXIF metadata fields
/// Expects GPS fields in standard EXIF format
pub fn extract_gps_from_metadata(
    gps_latitude: Option<&str>,
    gps_latitude_ref: Option<&str>,
    gps_longitude: Option<&str>,
    gps_longitude_ref: Option<&str>,
) -> Option<LocationData> {
    let lat_str = gps_latitude?;
    let lat_ref = gps_latitude_ref?;
    let lon_str = gps_longitude?;
    let lon_ref = gps_longitude_ref?;

    // Parse latitude and longitude (format: "37 deg 46' 26.40\" N")
    let lat = parse_gps_coordinate(lat_str)?;
    let lon = parse_gps_coordinate(lon_str)?;

    // Apply direction (N/S for latitude, E/W for longitude)
    let latitude = if lat_ref == "S" || lat_ref == "South" { -lat } else { lat };
    let longitude = if lon_ref == "W" || lon_ref == "West" { -lon } else { lon };

    Some(LocationData {
        latitude,
        longitude,
    })
}

/// Parses GPS coordinate string in DMS (degrees, minutes, seconds) format
/// Examples: "37 deg 46' 26.40\" N", "37 deg 46' 26.40\"", "37.7749", "37 46.44"
fn parse_gps_coordinate(coord_str: &str) -> Option<f64> {
    let cleaned = coord_str
        .replace("deg", "")
        .replace(['\'', '"'], " ")
        .trim()
        .to_string();

    let parts: Vec<&str> = cleaned.split_whitespace().collect();

    // Filter out directional letters (N, S, E, W) that might appear at the end
    let numeric_parts: Vec<&str> = parts
        .iter()
        .filter(|p| !matches!(p.to_uppercase().as_str(), "N" | "S" | "E" | "W"))
        .copied()
        .collect();

    match numeric_parts.len() {
        1 => {
            // Decimal format: "37.7749"
            numeric_parts[0].parse::<f64>().ok()
        }
        2 => {
            // Degrees and decimal minutes: "37 46.44"
            let degrees: f64 = numeric_parts[0].parse().ok()?;
            let minutes: f64 = numeric_parts[1].parse().ok()?;
            Some(degrees + minutes / 60.0)
        }
        3 => {
            // Degrees, minutes, seconds: "37 46 26.40"
            let degrees: f64 = numeric_parts[0].parse().ok()?;
            let minutes: f64 = numeric_parts[1].parse().ok()?;
            let seconds: f64 = numeric_parts[2].parse().ok()?;
            Some(degrees + minutes / 60.0 + seconds / 3600.0)
        }
        _ => None,
    }
}

/// Formats location data as a readable string
/// Example: "37.77N_122.42W"
pub fn format_location(loc: &LocationData) -> String {
    let lat_dir = if loc.latitude >= 0.0 { "N" } else { "S" };
    let lon_dir = if loc.longitude >= 0.0 { "E" } else { "W" };

    format!(
        "{:.2}{}_{}{}",
        loc.latitude.abs(),
        lat_dir,
        loc.longitude.abs(),
        lon_dir
    )
}

/// Formats a timestamp string into a human-readable format
/// Supports various EXIF timestamp formats
pub fn format_timestamp(datetime_str: &str) -> Option<String> {
    // Try common EXIF timestamp formats
    let formats = vec![
        "%Y:%m:%d %H:%M:%S",     // "2023:10:15 14:30:22"
        "%Y-%m-%d %H:%M:%S",     // "2023-10-15 14:30:22"
        "%Y%m%d_%H%M%S",         // "20231015_143022"
        "%Y:%m:%d",              // "2023:10:15"
        "%Y-%m-%d",              // "2023-10-15"
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
            // Format as YYYY-MM-DD
            return Some(dt.format("%Y-%m-%d").to_string());
        }
    }

    // Try date-only formats
    if let Some(date_only) = format_date_only(datetime_str) {
        return Some(date_only);
    }

    None
}

/// Formats date-only strings
fn format_date_only(date_str: &str) -> Option<String> {
    use chrono::NaiveDate;

    let formats = vec![
        "%Y:%m:%d",  // "2023:10:15"
        "%Y-%m-%d",  // "2023-10-15"
        "%Y%m%d",    // "20231015"
    ];

    for format in formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }

    None
}

/// Determines time of day from timestamp for optional enrichment
/// Returns: "morning", "afternoon", "evening", or "night"
pub fn get_time_of_day(datetime_str: &str) -> Option<&'static str> {
    let formats = vec![
        "%Y:%m:%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y%m%d_%H%M%S",
    ];

    for format in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
            let hour = dt.hour();
            return Some(match hour {
                5..=11 => "morning",
                12..=17 => "afternoon",
                18..=21 => "evening",
                _ => "night",
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gps_coordinate_decimal() {
        let result = parse_gps_coordinate("37.7749");
        assert!(result.is_some());
        assert!((result.unwrap() - 37.7749).abs() < 0.0001);
    }

    #[test]
    fn test_parse_gps_coordinate_dms() {
        let result = parse_gps_coordinate("37 46 26.40");
        assert!(result.is_some());
        // 37 + 46/60 + 26.40/3600 ≈ 37.7740
        assert!((result.unwrap() - 37.7740).abs() < 0.0001);
    }

    #[test]
    fn test_parse_gps_coordinate_deg_format() {
        let result = parse_gps_coordinate("37 deg 46' 26.40\"");
        assert!(result.is_some());
        assert!((result.unwrap() - 37.7740).abs() < 0.0001);
    }

    #[test]
    fn test_extract_gps_from_metadata() {
        let loc = extract_gps_from_metadata(
            Some("37 46 26.40"),
            Some("N"),
            Some("122 25 9.60"),
            Some("W"),
        );

        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert!((loc.latitude - 37.7740).abs() < 0.0001);
        assert!((loc.longitude + 122.4193).abs() < 0.0001); // Negative for West
    }

    #[test]
    fn test_format_location() {
        let loc = LocationData {
            latitude: 37.7749,
            longitude: -122.4194,
        };

        let formatted = format_location(&loc);
        // Should be approximately "37.77N_122.42W"
        assert!(formatted.contains("N"));
        assert!(formatted.contains("W"));
        assert!(formatted.contains("37.77"));
        assert!(formatted.contains("122.4"));
    }

    #[test]
    fn test_format_timestamp_exif_style() {
        let result = format_timestamp("2023:10:15 14:30:22");
        assert_eq!(result, Some("2023-10-15".to_string()));
    }

    #[test]
    fn test_format_timestamp_standard() {
        let result = format_timestamp("2023-10-15 14:30:22");
        assert_eq!(result, Some("2023-10-15".to_string()));
    }

    #[test]
    fn test_format_timestamp_compact() {
        let result = format_timestamp("20231015_143022");
        assert_eq!(result, Some("2023-10-15".to_string()));
    }

    #[test]
    fn test_format_timestamp_date_only() {
        let result = format_timestamp("2023:10:15");
        assert_eq!(result, Some("2023-10-15".to_string()));
    }

    #[test]
    fn test_get_time_of_day() {
        assert_eq!(
            get_time_of_day("2023:10:15 08:30:00"),
            Some("morning")
        );
        assert_eq!(
            get_time_of_day("2023:10:15 14:30:00"),
            Some("afternoon")
        );
        assert_eq!(
            get_time_of_day("2023:10:15 19:30:00"),
            Some("evening")
        );
        assert_eq!(get_time_of_day("2023:10:15 23:30:00"), Some("night"));
    }

    #[test]
    fn test_format_location_southern_hemisphere() {
        let loc = LocationData {
            latitude: -33.8688,  // Sydney
            longitude: 151.2093,
        };

        let formatted = format_location(&loc);
        assert!(formatted.contains("S"));
        assert!(formatted.contains("E"));
    }
}
