type Error = Box<dyn std::error::Error + Send + Sync>;
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
    let mut r = 0u64;
    let mut g = 0u64;
    let mut b = 0u64;

    // go through the pixels and calculate average color
    for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y).0; // Get pixel (R, G, B, A)
            r += pixel[0] as u64;
            g += pixel[1] as u64;
            b += pixel[2] as u64;
        }
    }

    // calculate the average
    let num_pixels = (width * height) as u64;
    r /= num_pixels;
    g /= num_pixels;
    b /= num_pixels;

    Ok(serenity::all::Color::from_rgb(r as u8, g as u8, b as u8))
}
