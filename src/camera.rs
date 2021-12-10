use xiapi_sys::*;

pub struct Camera {
    device_handle: HANDLE
}

pub fn open_device(dev_id: Option<u32>) -> Result<Camera, XI_RETURN>
{
    let mut device_handle: HANDLE = std::ptr::null_mut();
    let dev_id = dev_id.unwrap_or(0);
    let err = unsafe { xiapi_sys::xiOpenDevice(dev_id, &mut device_handle) };
    match err as u32{
        XI_RET::XI_OK => {Ok(Camera{device_handle})}
        _ => Err(err)
    }
}

impl Drop for Camera{
    fn drop(&mut self) {
        unsafe { xiapi_sys::xiCloseDevice(self.device_handle); }
    }
}