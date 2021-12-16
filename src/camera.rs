use crate::Image;
use std::ffi::CStr;
use xiapi_sys::XI_RET::XI_INVALID_ARG;
use xiapi_sys::*;

pub struct Camera {
    device_handle: HANDLE,
}

pub struct AcquisitionBuffer {
    camera: Camera,
}

pub fn open_device(dev_id: Option<u32>) -> Result<Camera, XI_RETURN> {
    let mut device_handle: HANDLE = std::ptr::null_mut();
    let dev_id = dev_id.unwrap_or(0);
    let err = unsafe { xiapi_sys::xiOpenDevice(dev_id, &mut device_handle) };
    match err as u32 {
        XI_RET::XI_OK => Ok(Camera { device_handle }),
        _ => Err(err),
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            xiapi_sys::xiCloseDevice(self.device_handle);
        }
    }
}

impl Camera {
    pub fn start_acquisition(self) -> Result<AcquisitionBuffer, XI_RETURN> {
        let err = unsafe { xiapi_sys::xiStartAcquisition(self.device_handle) };
        match err as u32 {
            XI_RET::XI_OK => Ok(AcquisitionBuffer { camera: self }),
            _ => Err(err),
        }
    }

    unsafe fn param_float(&self, param: &[u8]) -> Result<f32, XI_RETURN> {
        let mut result: f32 = 0.0;
        let param_c = match CStr::from_bytes_with_nul(param) {
            Ok(c) => c,
            Err(_) => return Err(XI_INVALID_ARG as XI_RETURN),
        };
        let err = xiapi_sys::xiGetParamFloat(self.device_handle, param_c.as_ptr(), &mut result);
        match err as u32 {
            XI_RET::XI_OK => Ok(result),
            _ => Err(err),
        }
    }

    unsafe fn set_param_float(&mut self, param: &[u8], value: f32) -> Result<(), XI_RETURN> {
        let param_c = match CStr::from_bytes_with_nul(param) {
            Ok(c) => c,
            Err(_) => return Err(XI_INVALID_ARG as XI_RETURN),
        };
        let err = xiapi_sys::xiSetParamFloat(self.device_handle, param_c.as_ptr(), value);
        match err as u32 {
            XI_RET::XI_OK => Ok(()),
            _ => Err(err),
        }
    }

    pub fn exposure(&self) -> Result<f32, XI_RETURN> {
        unsafe { self.param_float(XI_PRM_EXPOSURE) }
    }

    pub fn set_exposure(&mut self, value: f32) -> Result<(), XI_RETURN> {
        unsafe { self.set_param_float(XI_PRM_EXPOSURE, value) }
    }

    pub fn gain(&self) -> Result<f32, XI_RETURN> {
        unsafe { self.param_float(XI_PRM_GAIN) }
    }

    pub fn set_gain(&mut self, value: f32) -> Result<(), XI_RETURN> {
        unsafe { self.set_param_float(XI_PRM_GAIN, value) }
    }
}

impl AcquisitionBuffer {
    pub fn stop_acquisition(self) -> Result<Camera, XI_RETURN> {
        let err = unsafe { xiapi_sys::xiStopAcquisition(self.camera.device_handle) };
        match err as u32 {
            XI_RET::XI_OK => Ok(self.camera),
            _ => Err(err),
        }
    }

    pub fn next_image<T>(&self, timeout: Option<u32>) -> Result<Image<T>, XI_RETURN> {
        let timeout = timeout.unwrap_or(u32::MAX);
        let mut image = Image::new();
        unsafe {
            xiapi_sys::xiGetImage(self.camera.device_handle, timeout, &mut image.xi_img);
        }
        Ok(image)
    }
}
