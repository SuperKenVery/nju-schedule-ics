#[derive(Debug)]
pub struct GeoLocation {
    latitude: f64,
    longitude: f64,
}

impl GeoLocation {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn to_ical_str(&self) -> String {
        format!("{};{}", self.latitude, self.longitude)
    }

    pub fn to_apple_location_str(&self) -> String {
        format!("geo:{},{}", self.latitude, self.longitude)
    }
}
