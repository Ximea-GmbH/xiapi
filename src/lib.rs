pub use camera::*;

pub mod camera;

#[cfg(test)]
mod tests {
    use xiapi_sys::XI_RETURN;

    use crate::open_device;

    #[test]
    fn start_stop_acquisition() -> Result<(), XI_RETURN> {
        let cam = open_device(None)?;
        let acq = cam.start_acquisition()?;
        acq.stop_acquisition()?;
        Ok(())
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
