/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */

use std::marker::PhantomData;
use std::mem::{MaybeUninit, size_of};

use xiapi_sys::XI_IMG;

/// An Image as it is captured by the camera.
pub struct Image<T> {
    pub(crate) xi_img: XI_IMG,
    pix_type: std::marker::PhantomData<T>,
}

impl<T> Image<T> {
    /// Creates a new image.
    ///
    /// The returned image does not contain any data and image metadata are all empty or zero.
    // FIXME: This function should not be public as the only way to get an image should be through the camera
    // FIXME: It should probably also be unsafe.
    pub fn new() -> Self {
        let image = unsafe {
            let mut img = MaybeUninit::<XI_IMG>::zeroed().assume_init();
            img.size = size_of::<XI_IMG>() as u32;
            img
        };
        Self {
            xi_img: image,
            pix_type: PhantomData,
        }
    }

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
}

impl<T> Default for Image<T> {
    fn default() -> Self {
        Self::new()
    }
}
