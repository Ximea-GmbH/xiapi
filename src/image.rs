/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */

use std::mem::size_of;

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
    ///
    /// # Safety
    /// You must ensure that the image was properly initialized and contains valid pixel values
    /// before calling this method.
    pub unsafe fn pixel(&self, x: usize, y: usize) -> Option<&T> {
        let buffer = self.xi_img.bp as *const u8;
        // stride is the total length of a row in bytes
        let stride = self.xi_img.width as usize * size_of::<T>() + self.xi_img.padding_x as usize;
        let offset = (stride * y) + (x * size_of::<T>());
        let pixel_pointer = buffer.add(offset) as *const T;
        pixel_pointer.as_ref()
    }
}

