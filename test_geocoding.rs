// Simple test for geocoding functionality
use nameback_core::geocoding::reverse_geocode;

fn main() {
    println!("Testing geocoding functionality...\n");

    // Test some known coordinates
    let test_cases = vec![
        (47.6062, -122.3321, "Seattle, WA"),      // Seattle
        (40.7128, -74.0060, "New York, NY"),      // NYC
        (37.7749, -122.4194, "San Francisco, CA"), // SF
        (51.5074, -0.1278, "London, UK"),         // London
        (35.6762, 139.6503, "Tokyo, Japan"),      // Tokyo
    ];

    for (lat, lon, expected_area) in test_cases {
        print!("Testing {:.4}°, {:.4}° (near {})... ", lat, lon, expected_area);

        match reverse_geocode(lat, lon) {
            Some(location) => {
                println!("✓ Got: {}", location);
            }
            None => {
                println!("✗ Failed to geocode");
            }
        }

        // Sleep briefly to respect rate limits
        std::thread::sleep(std::time::Duration::from_millis(1100));
    }

    println!("\nTest complete!");
}