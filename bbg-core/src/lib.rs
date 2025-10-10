type Error = Box<dyn std::error::Error + Send + Sync>;
#[derive(Debug, Clone, Copy)]
// use this struct instead of serenity color
pub struct AverageColor {
    red: u8,
    green: u8,
    blue: u8,
}
impl AverageColor {
    fn to_embed_color(&self) -> Result<serenity::all::Color, Error> {
        Ok(serenity::all::Color::from_rgb(self.red, self.green, self.blue))
    }
}
pub fn calculate_tetrio_level(xp: f64) -> f64 {
    let xp =
        (xp / 500.0).powf(0.6) + (xp / (5000.0 + f64::max(0.0, xp - 4000000.0) / 5000.0)) + 1.0;
    xp.trunc()
}

// thia function was brought to you by chatgpt
// i mean it works? thankfully no need to read docs lol (bad for me no lie)
#[cfg(feature = "discord")]
pub async fn get_avatar_color(url: &str) -> Result<serenity::all::Color, Error> {
    use image::GenericImageView;
    let img_bytes = &reqwest::get(url).await?.bytes().await?;

    let img = image::load_from_memory(&img_bytes)?;
    let (width, height) = img.dimensions();
    let mut red = 0u64;
    let mut green = 0u64;
    let mut blue = 0u64;

    // go through the pixels and calculate average color
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y).0; // Get pixel (R, G, B, A)
            red     += pixel[0] as u64;
            green   += pixel[1] as u64;
            blue    += pixel[2] as u64;
        }
    }

    // calculate the average
    let num_pixels = (width * height) as u64;
    red     /= num_pixels;
    green   /= num_pixels;
    blue    /= num_pixels;

    let result = AverageColor {
        red: red as u8,
        green: green as u8,
        blue: blue as u8,
    };

    let result = result.to_embed_color();

    Ok(result?)
}
