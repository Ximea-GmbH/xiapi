use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use xiapi_sys::XI_IMG;

pub struct Image<T> {
    pub(crate) xi_img: XI_IMG,
    pix_type: std::marker::PhantomData<T>,
}

impl<T> Image<T> {
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

    pub unsafe fn pixel(&self, x: usize, y: usize) -> Option<&T> {
        let buffer = self.xi_img.bp as *const u8;
        // stride is the total length of a row in bytes
        let stride = self.xi_img.width as usize * size_of::<T>() + self.xi_img.padding_x as usize;
        let offset = (stride * y) + (x * size_of::<T>());
        let pixel_pointer = buffer.add(offset) as *const T;
        pixel_pointer.as_ref()
    }
}

impl<T> Default for Image<T> {
    fn default() -> Self {
        Self::new()
    }
}
