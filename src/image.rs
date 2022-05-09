/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */

use std::mem::size_of;
use std::slice::from_raw_parts;

#[cfg(feature = "image")]
use image::{ImageBuffer, Pixel};

use xiapi_sys::XI_IMG;

/// An Image as it is captured by the camera.
pub struct Image<'a, T> {
    pub(crate) xi_img: XI_IMG,
    pub(crate) pix_type: std::marker::PhantomData<&'a T>,
}


impl<'a, T> Image<'a, T> {
    /// Get a Pixel from the image.
    ///
    /// # Arguments
    ///
    /// * `x`: Horizontal coordinate of the requested pixel.
    /// * `y`: Vertical coordinate of the requested pixel.
    ///
    /// returns: Option<&T> A reference to the pixel
    pub fn pixel(&self, x: usize, y: usize) -> Option<&T> {
        let buffer = self.xi_img.bp as *const u8;
        // Check if uninitialized
        if buffer.is_null() {
            return None;
        }
        // Bounds check
        if x >= self.xi_img.width as usize || y >= self.xi_img.height as usize {
            return None;
        }
        // stride is the total length of a row in bytes
        let stride = self.xi_img.width as usize * size_of::<T>() + self.xi_img.padding_x as usize;
        let offset = (stride * y) + (x * size_of::<T>());
        unsafe {
            let pixel_pointer = buffer.add(offset) as *const T;
            pixel_pointer.as_ref()
        }
    }

    /// Get the width of this image in pixels
    pub fn width(&self) -> u32 {
        self.xi_img.width
    }

    /// Get the height of this image
    pub fn height(&self) -> u32 {
        self.xi_img.height
    }

    /// Get the raw image data as a slice
    pub fn data(&'a self) -> &'a [T] {
        unsafe {
            from_raw_parts(self.xi_img.bp as *const T, self.xi_img.bp_size as usize)
        }
    }
}

#[cfg(feature = "image")]
impl<P> From<Image<'_, P::Subpixel>> for ImageBuffer<P, Vec<P::Subpixel>>
where
    P: Pixel,
{
    /// Converts the image to an [ImageBuffer]
    /// ```
    /// # #[serial_test::file_serial]
    /// # fn main() -> Result<(), xiapi_sys::XI_RETURN>{
    /// # use image::{ImageBuffer, Luma};
    /// # let cam = xiapi::open_device(None)?;
    /// # let buffer = cam.start_acquisition()?;
    /// let image = buffer.next_image::<u8>(None)?;
    /// let image_buffer = ImageBuffer::<Luma<u8>,_>::from(image);
    /// image_buffer.save("test.jpg");
    /// # Ok(())
    /// # }
    /// ```

    fn from(image: Image<P::Subpixel>) -> Self {
        let data = Vec::from(image.data());
        match Self::from_raw(image.width(), image.height(), data) {
            None => panic!("Failed to create image from raw pointer"),
            Some(buffer) => {buffer}
        }
    }
}

