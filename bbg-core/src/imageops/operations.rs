use crate::Error;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "discord", derive(poise::ChoiceParameter))]
pub enum ImageOrientation {
    #[cfg_attr(feature = "discord", name = "horizontally")]
    Horizontal,
    #[cfg_attr(feature = "discord", name = "vertically")]
    Vertical,
}

#[derive(Debug, Default)]
pub struct ImageOperations {
    pub blur: Option<f32>,
    pub orientation: Option<ImageOrientation>,
    pub grayscale: bool,
}

impl ImageOperations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_identity(&self) -> bool {
        self.blur.is_none() && self.orientation.is_none() && !self.grayscale
    }

    pub fn apply_to(&self, image: DynamicImage) -> Result<DynamicImage, Error> {
        let mut img = image;

        if let Some(orientation) = self.orientation {
            match orientation {
                ImageOrientation::Horizontal => img = img.fliph(),
                ImageOrientation::Vertical => img = img.flipv(),
            }
        }

        if let Some(blur_amount) = self.blur {
            img = img.fast_blur(blur_amount);
        }

        if self.grayscale {
            img = img.grayscale();
        }

        Ok(img)
    }
}

#[derive(Debug)]
pub struct ImageProcessor {
    url: String,
    operations: ImageOperations,
}

impl ImageProcessor {
    pub fn new(url: String) -> Self {
        Self {
            url,
            operations: ImageOperations::new(),
        }
    }

    /// Blurs an image using imageops' fast_blur method
    pub fn blur(mut self, blur: Option<f32>) -> Self {
        if let Some(amount) = blur {
            self.operations.blur = Some(amount);
        }
        self
    }

    /// Flips an image
    pub fn flip(mut self, orientation: Option<ImageOrientation>) -> Self {
        if let Some(orient) = orientation {
            self.operations.orientation = Some(orient);
        }
        self
    }

    /// Self explainatory
    pub fn grayscale(mut self, grayscale: Option<bool>) -> Self {
        if grayscale.is_some_and(|g| g) {
            self.operations.grayscale = true;
        }
        self
    }

    /// Call to process the image
    /// Returns raw bytes (in `Vec<u8>`)  
    /// Like this:  
    /// ```
    ///    let result = ImageProcessor::new(url)
    ///      .blur(blur)
    ///      .flip(orientation)
    ///      .grayscale(grayscale)
    ///      .process()
    ///    .await?;
    /// ```

    pub async fn process(self) -> Result<Vec<u8>, Error> {
        if self.operations.is_identity() {
            return Err("no operations specified".into());
        }

        let img_bytes = reqwest::get(&self.url).await?.bytes().await?;

        let processed_image = tokio::task::spawn_blocking(move || {
            let image = image::load_from_memory(&img_bytes)?;
            self.operations.apply_to(image)
        })
        .await??;

        // Convert to PNG bytes
        Ok(
            tokio::task::spawn_blocking(move || -> Result<Vec<u8>, Error> {
                let mut bytes = Vec::new();
                processed_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
                Ok(bytes)
            })
            .await??,
        )
    }
}
