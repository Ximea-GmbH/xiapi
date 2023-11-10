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
        let nb_channels = self.nb_channels();
        let stride = self.xi_img.width as usize * size_of::<T>() * nb_channels
            + self.xi_img.padding_x as usize;
        let offset = (stride * y) + (x * size_of::<T>() * nb_channels);
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

    /// Format of image data
    pub fn format(&self) -> xiapi_sys::XI_IMG_FORMAT::Type {
        self.xi_img.frm
    }

    /// Frame number
    pub fn nframe(&self) -> u32 {
        self.xi_img.nframe
    }

    /// Image black level
    pub fn black_level(&self) -> u32 {
        self.xi_img.black_level
    }

    /// Number of extra bytes provided at the end of each line for alignment
    pub fn padding_x(&self) -> u32 {
        self.xi_img.padding_x
    }

    /// Horizontal offset from the origin of the sensor to the first pixel in this image
    pub fn absolute_offset_x(&self) -> u32 {
        self.xi_img.AbsoluteOffsetX
    }

    /// Vertical offset from the origin of the sensor to the first line in this image
    pub fn absolute_offset_y(&self) -> u32 {
        self.xi_img.AbsoluteOffsetY
    }

    /// Current format of the pixels on transport layer
    pub fn transport_format(&self) -> xiapi_sys::XI_IMG_FORMAT::Type {
        self.xi_img.transport_frm
    }

    /// Horizontal downsampling
    pub fn downsampling_x(&self) -> u32 {
        self.xi_img.DownsamplingX
    }

    /// Vertical downsampling
    pub fn downsampling_y(&self) -> u32 {
        self.xi_img.DownsamplingY
    }

    /// Exposure time for this image in us
    pub fn exposure_time_us(&self) -> u32 {
        self.xi_img.exposure_time_us
    }

    /// Aquisition Frame Number. Reset only on acquisition start.
    pub fn acq_nframe(&self) -> u32 {
        self.xi_img.acq_nframe
    }

    /// Image user data which can be set using [Camera::set_image_user_data]
    pub fn image_user_data(&self) -> u32 {
        self.xi_img.image_user_data
    }

    /// Raw 64-bit timestamp from the camera. Interpretation of this value differs between camera series.
    /// xiQ, xiD: 40-bit microsecond number - (overlaps after 305 hours)
    /// xiC, xiB, xiT, xiX: 64-bit 4 nanosecond number (overlaps after 2339 years)
    pub fn timestamp_raw(&self) -> u64 {
        let high = self.xi_img.tsSec as u64;
        let low = self.xi_img.tsUSec as u64;
        (high << 32) | low
    }

    /// Get the raw image data as a slice.
    pub fn data(&'a self) -> &'a [T] {
        unsafe {
            if self.xi_img.bp_size != 0 {
                let length = self.xi_img.bp_size as usize / size_of::<T>();
                from_raw_parts(self.xi_img.bp as *const T, length)
            }
            else {
                let length = self.xi_img.width as usize * self.xi_img.height as usize * self.nb_channels();
                from_raw_parts(self.xi_img.bp as *const T, length)
            }
        }
    }

    fn nb_channels(&self) -> usize
    {
        match self.xi_img.frm {
            xiapi_sys::XI_IMG_FORMAT::XI_MONO8  => 1,
            xiapi_sys::XI_IMG_FORMAT::XI_MONO16 => 1,
            xiapi_sys::XI_IMG_FORMAT::XI_RAW8   => 1,
            xiapi_sys::XI_IMG_FORMAT::XI_RAW16  => 1,
            xiapi_sys::XI_IMG_FORMAT::XI_RGB24  => 3,
            xiapi_sys::XI_IMG_FORMAT::XI_RGB32  => 4,

            _ => 0,
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
            Some(buffer) => buffer,
        }
    }
}
