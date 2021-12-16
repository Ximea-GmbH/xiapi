pub use camera::*;
pub use image::*;

pub mod camera;
pub mod image;

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
        let test = unsafe { img.pixel(100, 100) }.unwrap_or_else(|| {
            panic!("Pixel value was invalid!");
        });
        print!("Pixel Value was read as {}", *test);
        Ok(())
    }
}
