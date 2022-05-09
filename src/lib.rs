/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */

//! High level Rust bindings for the XIMEA camera API
//!
//! This crate provides a common interface for all XIMEA cameras.
//! It is a higher level wrapper of the xiapi-sys crate which is generated automatically
//! (via bindgen) from the XIMEA C API.

#![warn(missing_docs)]

pub use self::camera::open_device;
pub use self::camera::AcquisitionBuffer;
pub use self::camera::Camera;
pub use self::image::Image;

mod camera;
mod image;

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use serial_test::serial;
    use xiapi_sys::XI_RETURN;

    use crate::open_device;

    #[test]
    #[serial]
    fn start_stop_acquisition() -> Result<(), XI_RETURN> {
        let cam = open_device(None)?;
        let acq = cam.start_acquisition()?;
        acq.stop_acquisition()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn set_get_exposure() -> Result<(), XI_RETURN> {
        let mut cam = open_device(None)?;
        cam.set_exposure(12_345.0)?;
        let exp = cam.exposure()?;
        assert_abs_diff_eq!(exp, 12_345.0, epsilon = 10.0);
        Ok(())
    }

    #[test]
    #[serial]
    fn default_gain() -> Result<(), XI_RETURN> {
        let cam = open_device(None)?;
        let gain = cam.gain()?;
        assert_eq!(gain, 0.0);
        Ok(())
    }

    #[test]
    #[serial]
    fn get_image() -> Result<(), XI_RETURN> {
        let cam = open_device(None)?;
        let acq = cam.start_acquisition()?;
        let img = acq.next_image::<u8>(None)?;
        let test_pix = img.pixel(100, 0).unwrap_or_else(|| {
            panic!("Pixel value was invalid!");
        });
        let test_data = img.data();
        let test_data_pix = test_data.get(100).unwrap_or_else(|| {
            panic!("Out of bounds error on image data!");
        });
        assert_eq!(test_pix, test_data_pix);

        Ok(())
    }

}
