pub mod imageops;
pub mod jobs;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone, Copy)]
// use this struct instead of serenity color
pub struct AverageColor {
    red: u8,
    green: u8,
    blue: u8,
}
impl AverageColor {
    #[cfg(feature = "discord")]
    pub fn to_embed_color(&self) -> serenity::all::Color {
        serenity::all::Color::from_rgb(self.red, self.green, self.blue)
    }

    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
    pub async fn from_image_url(url: &str) -> Result<AverageColor, Error> {
        use image::{GenericImageView, load_from_memory};
        let img_bytes = &reqwest::get(url).await?.bytes().await?;

        let img = load_from_memory(img_bytes)?;
        let (width, height) = img.dimensions();
        let mut red = 0u64;
        let mut green = 0u64;
        let mut blue = 0u64;

        // go through the pixels and calculate average color
        for x in 0..width {
            for y in 0..height {
                let pixel = img.get_pixel(x, y).0; // Get pixel (R, G, B, A)
                red += pixel[0] as u64;
                green += pixel[1] as u64;
                blue += pixel[2] as u64;
            }
        }

        // calculate the average
        let num_pixels = (width * height) as u64;
        red /= num_pixels;
        green /= num_pixels;
        blue /= num_pixels;

        Ok(AverageColor::new(red as u8, green as u8, blue as u8))
    }
}
pub fn calculate_tetrio_level(xp: f64) -> f64 {
    let xp =
        (xp / 500.0).powf(0.6) + (xp / (5000.0 + f64::max(0.0, xp - 4000000.0) / 5000.0)) + 1.0;
    xp.trunc()
}

pub async fn pi() -> String {
    use rand::Rng;
    let mut pi_string = format!("{:.15}", std::f64::consts::PI); // get Pi to 15 decimal places

    // VERY SMALL CHANCE to mess up a digit
    if rand::rng().random_bool(0.000000001454) {
        let digits: Vec<char> = pi_string.chars().collect();
        let mut rng = rand::rng();

        // pick a random index after the decimal point (skip '3' and '.')
        let idx = rng.random_range(2..digits.len());
        let new_digit = rng.random_range(0..10).to_string().chars().next().unwrap();

        let mut new_pi_string = digits.clone();
        new_pi_string[idx] = new_digit;
        pi_string = new_pi_string.iter().collect();
    }

    pi_string
}

pub async fn get_random_ipv4() -> String {
    use rand::prelude::*;
    let mut rng = rand::rngs::StdRng::from_os_rng();

    let octet1: u8 = rng.random_range(0..=255);
    let octet2: u8 = rng.random_range(0..=255);
    let octet3: u8 = rng.random_range(0..=255);
    let octet4: u8 = rng.random_range(0..=255);

    format!("{}.{}.{}.{}", octet1, octet2, octet3, octet4)
}
