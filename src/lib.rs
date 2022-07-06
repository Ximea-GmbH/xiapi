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
pub use self::camera::open_device_manual_bandwidth;
pub use self::camera::AcquisitionBuffer;
pub use self::camera::Camera;
pub use self::image::Image;
pub use self::roi::Roi;

mod camera;
mod image;
mod roi;

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use serial_test::serial;
    use xiapi_sys::XI_RETURN;
    use xiapi_sys::XI_GAIN_SELECTOR_TYPE::XI_GAIN_SELECTOR_ALL;
    use xiapi_sys::XI_DOWNSAMPLING_VALUE::XI_DWN_1x1;
    use xiapi_sys::XI_DOWNSAMPLING_TYPE::*;
    //use xiapi_sys::XI_TEST_PATTERN_GENERATOR::*;
    use xiapi_sys::XI_TEST_PATTERN::*;
    use crate::Roi;

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
        cam.set_exposure_burst_count(1)?;
        cam.set_exposure(12_345.0)?;
        let exp = cam.exposure()?;
        assert_abs_diff_eq!(exp, 12_345.0, epsilon = 10.0);
        Ok(())
    }

    #[test]
    #[serial]
    fn default_gains() -> Result<(), XI_RETURN> {
        let mut cam = open_device(None)?;
        cam.set_gain_selector(XI_GAIN_SELECTOR_ALL)?;
        let gain_all = cam.gain()?;
        assert_eq!(gain_all, 0.0);
        Ok(())
    }

    #[test]
    #[serial]
    fn downsampling_defaults() -> Result<(), XI_RETURN> {
        let mut cam = open_device(None)?;
        let default_type = cam.downsampling_type()?;
        assert_eq!(default_type, XI_BINNING);
        let default_value = cam.downsampling()?;
        assert_eq!(default_value, XI_DWN_1x1);
        cam.set_downsampling_type(XI_SKIPPING)?;
        let skipping_value = cam.downsampling()?;
        assert_eq!(skipping_value, XI_DWN_1x1);
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

    #[test]
    #[serial]
    fn test_pattern_defaults() -> Result<(), XI_RETURN> {
        let cam = open_device(None)?;
        //let generator = cam.test_pattern_generator_selector()?;
        //assert_eq!(generator, XI_TESTPAT_GEN_FPGA);
        let pattern = cam.test_pattern()?;
        assert_eq!(pattern, XI_TESTPAT_OFF);
        Ok(())
    }

    #[test]
    #[serial]
    fn get_increment() -> Result<(), XI_RETURN> {
        let mut cam = open_device(None)?;
        let increment = cam.width_increment()?;
        println!("{}", increment);
        Ok(())
    }

    #[test]
    #[serial]
    fn set_get_roi() -> Result<(), XI_RETURN> {
        let mut cam = open_device(None)?;
        let roi = Roi{
            offset_x: cam.offset_x_increment()?,
            offset_y: cam.offset_y_increment()?,
            width: cam.width_increment()?,
            height: cam.height_increment()?,
        };
        let roi_actual = cam.set_roi(&roi)?;
        assert_eq!(roi.width, roi_actual.width);

        let get_roi = cam.roi()?;
        assert_eq!(roi.width, get_roi.width);

        Ok(())
    }
}
