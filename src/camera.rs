/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::mem::size_of;
use std::os::raw::c_char;

use paste::paste;
use xiapi_sys::*;

use crate::Image;

/// This macro is used to generate getters and setters for xiAPI parameters.
/// The parameters are specified using the following syntax: \[mut\] <ParamName>: <Type>
/// Documentation on the parameter will be added to the getter.
/// Generic documentation is always added to the setter.
///
/// ## Examples:
///
/// ```ignore
/// param!(
///     mut exposure: f32;
/// )
/// ```
macro_rules! param {
    // This rule follows the Incremental TT muncher pattern.
    () => {};
    // For mutable parameters:
    (
        $(#[doc = $doc:expr])*
        mut $prm:ident : $type:ty;
        $($tail:tt)*
    ) => {
        paste! {
            // Generate a getter with custom documentation
            $(#[doc = $doc])*
            pub fn $prm(&self) -> Result<$type, XI_RETURN>{
                unsafe {self.param([<XI_PRM_ $prm:upper>]) }
             }
            // Generate a setter
            // TODO: Customizable documentation for setters
            #[doc = "Set the `" $prm "` parameter. See also [Self::" $prm "()]"]
            pub fn [<set_ $prm>](& mut self, value: $type ) -> Result<(), XI_RETURN>{
                unsafe {self.set_param([<XI_PRM_ $prm:upper>], value)}
            }
            param!($($tail)*);
        }
    };
    // For immutable parameters
    (
        $(#[doc = $doc:expr])*
        $prm:ident : $type:ty;
        $($tail:tt)*
    ) => {
        paste! {
            // Generate a getter with custom documentation
            $(#[doc = $doc])*
            pub fn $prm( &self) -> Result < $type, XI_RETURN >{
                unsafe {self.param(paste ! ([ < XI_PRM_ $prm: upper > ]))}
            }
            param!($($tail)*);
        }
    };
}
/// Connected and initialized XIMEA camera.
///
/// Must be mutable to allow changing any parameters. A non-mutable Camera can be used from
/// multiple threads or processes safely.
pub struct Camera {
    device_handle: HANDLE,
}

/// Buffer that is used by the camera to transfer images to the host system.
///
/// The AcquisitionBuffer is the primary way to communicate with the camera while it is actively
/// acquiring images.
/// It is generated when the image acquisition is started and destroyed when the image acquisition
/// is stopped.
///
/// **Important difference to the C/C++ xiAPI:**
/// The AcquisitionBuffer temporarily consumes the Camera during acquisition, to prevent any
/// interactions that may change parameters that are fixed while the image acquisition is running.
/// Trying to change an parameter that is not changeable during acquisition is therefore an error at
/// compile time (as opposed to runtime in C/C++).
pub struct AcquisitionBuffer {
    camera: Camera,
}

/// Initializes a camera and returns it.
///
/// If successful, this function returns a Camera object that represents the camera which was
/// initialized.
/// If an error occurs, the Result contains the error code.
///
/// It is possible but not recommended to open the same camera from different processes at the same
/// time.
/// The device is automatically closed when the Camera object is dropped.
///
/// The automatic bandwidth calculation is enabled by default when using this method.
///
/// # Arguments
///
/// * `dev_id`: The device ID for the device to be initialized. Usually device IDs are sequential
/// and start at 0 for the first device in the system. Default value: 0
///
/// # Examples
///
/// ```
/// # #[serial_test::file_serial]
/// # fn main() -> Result<(), xiapi_sys::XI_RETURN>{
///     let mut cam = xiapi::open_device(None)?;
///     cam.set_exposure(10000 as f32);
///     // Do more stuff with the camera ...
/// #   Ok(())
/// # }
/// ```
pub fn open_device(dev_id: Option<u32>) -> Result<Camera, XI_RETURN> {
    let mut device_handle: HANDLE = std::ptr::null_mut();
    let dev_id = dev_id.unwrap_or(0);
    let err = unsafe { xiapi_sys::xiOpenDevice(dev_id, &mut device_handle) };
    match err as XI_RET::Type {
        XI_RET::XI_OK => Ok(Camera { device_handle }),
        _ => Err(err),
    }
}

/// Initialize the camera with the given bandwidth and return it.
///
/// If successful, this function returns a Camera object that represents the camera which was
/// initialized.
/// If an error occurs, the Result contains the error code.
///
/// The automatic bandwidth measurement is disabled when using this method. This can lead to faster device initialization.
///
/// # Arguments
///
/// *`dev_id`: The device ID for the device to be initialized. Usually device IDs are sequential
/// and start at 0 for the first device in the system. Default value: 0
/// *`bandwidth`: Transport layer bandwidth for this camera in MBit/s
///
/// # Examples
///
/// ```
/// # #[serial_test::file_serial]
/// # fn main() -> Result<(), xiapi_sys::XI_RETURN>{
///     let mut cam = xiapi::open_device_manual_bandwidth(None, 1000)?;
///     cam.set_exposure(10000 as f32);
///     // Do more stuff with the camera ...
/// #   Ok(())
/// # }
/// ```
pub fn open_device_manual_bandwidth(dev_id: Option<u32>, bandwidth: i32) -> Result<Camera, XI_RETURN>{
    let cam = unsafe {
        let bandwidth_param_c = match CStr::from_bytes_with_nul(XI_PRM_AUTO_BANDWIDTH_CALCULATION) {
            Ok(c) => c,
            Err(_) => return Err(XI_RET::XI_INVALID_ARG as XI_RETURN),
        };
        match i32::set_param(std::ptr::null_mut(), bandwidth_param_c.as_ptr(), XI_SWITCH::XI_OFF as i32) as u32{
            XI_RET::XI_OK => {}
            err => return Err(err as i32)
        };

        let cam = open_device(dev_id);
        match i32::set_param(std::ptr::null_mut(), bandwidth_param_c.as_ptr(), XI_SWITCH::XI_ON as i32) as u32{
            XI_RET::XI_OK => {}
            _ => panic!("Could not enable auto bandwidth calculation!")
        }
        cam
    };
    match cam  {
        Ok(mut cam) => {
            cam.set_limit_bandwidth(bandwidth)?;
            Ok(cam)
        }
        Err(err) => Err(err)
    }

}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            xiapi_sys::xiCloseDevice(self.device_handle);
        }
    }
}

trait ParamType: Default {
    unsafe fn get_param(
        handle: xiapi_sys::HANDLE,
        prm: *const std::os::raw::c_char,
        value: &mut Self,
    ) -> XI_RETURN;
    unsafe fn set_param(
        handle: xiapi_sys::HANDLE,
        prm: *const std::os::raw::c_char,
        value: Self,
    ) -> XI_RETURN;
}

impl ParamType for f32 {
    unsafe fn get_param(handle: HANDLE, prm: *const c_char, value: &mut Self) -> XI_RETURN {
        xiapi_sys::xiGetParamFloat(handle, prm, value)
    }

    unsafe fn set_param(handle: HANDLE, prm: *const c_char, value: Self) -> XI_RETURN {
        xiapi_sys::xiSetParamFloat(handle, prm, value)
    }
}

impl ParamType for i32 {
    unsafe fn get_param(handle: HANDLE, prm: *const c_char, value: &mut Self) -> XI_RETURN {
        xiapi_sys::xiGetParamInt(handle, prm, value)
    }

    unsafe fn set_param(handle: HANDLE, prm: *const c_char, value: Self) -> XI_RETURN {
        xiapi_sys::xiSetParamInt(handle, prm, value)
    }
}

impl ParamType for u32 {
    // Selectors in xiAPI are defined as unsigned int, but treated as if they were signed
    unsafe fn get_param(handle: HANDLE, prm: *const c_char, value: &mut Self) -> XI_RETURN {
        xiapi_sys::xiGetParamInt(handle, prm, value as *mut u32 as *mut i32)
    }

    unsafe fn set_param(handle: HANDLE, prm: *const c_char, value: Self) -> XI_RETURN {
        xiapi_sys::xiSetParamInt(handle, prm, value as i32)
    }
}

impl Camera {
    /// Starts the image acquisition on this camera
    ///
    /// This function creates the AcquisitionBuffer and tells the camera to start streaming data
    /// to this buffer.
    /// The camera is temporarily consumed by the AcquisitionBuffer, so you can only interact with
    /// it through the AcquisitionBuffer.
    ///
    /// # Examples
    /// ```
    /// # #[serial_test::file_serial]
    /// # fn main() -> Result<(), xiapi_sys::XI_RETURN> {
    ///     let cam = xiapi::open_device(None)?;
    ///     let buffer = cam.start_acquisition()?;
    ///     let image = buffer.next_image::<u8>(None)?;
    ///     // Do something with the image;
    ///     let cam = buffer.stop_acquisition()?;
    /// #   Ok(())
    /// # }
    pub fn start_acquisition(self) -> Result<AcquisitionBuffer, XI_RETURN> {
        let err = unsafe { xiapi_sys::xiStartAcquisition(self.device_handle) };
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(AcquisitionBuffer { camera: self }),
            _ => Err(err),
        }
    }

    unsafe fn set_param<T: ParamType>(&mut self, param: &[u8], value: T) -> Result<(), XI_RETURN> {
        let param_c = match CStr::from_bytes_with_nul(param) {
            Ok(c) => c,
            Err(_) => return Err(XI_RET::XI_INVALID_ARG as XI_RETURN),
        };
        let err = T::set_param(self.device_handle, param_c.as_ptr(), value);
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(()),
            _ => Err(err),
        }
    }

    unsafe fn param<T: ParamType>(&self, param: &[u8]) -> Result<T, XI_RETURN> {
        let mut value = T::default();
        let param_c = match CStr::from_bytes_with_nul(param) {
            Ok(c) => c,
            Err(_) => return Err(XI_RET::XI_INVALID_ARG as XI_RETURN),
        };
        let err = T::get_param(self.device_handle, param_c.as_ptr(), &mut value);
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(value),
            _ => Err(err),
        }
    }

    param! {
        /// Current exposure time in microseconds.
        mut exposure: f32;

        /// Sets the number of times of exposure in one frame.
        mut exposure_burst_count: i32;

        /// Set the gain in dB.
        /// If the camera has more than one type of gain, you can use [Self::set_gain_selector()] to
        /// select a gain.
        mut gain: f32;

        /// The currently selected type of gain for [Self::gain()] and [Self::set_gain()]
        mut gain_selector: XI_GAIN_SELECTOR_TYPE::Type;

        /// Changes image resolution by binning or skipping
        mut downsampling: XI_DOWNSAMPLING_VALUE::Type;

        /// Changes the downsampling type between binning and skipping
        mut downsampling_type: XI_DOWNSAMPLING_TYPE::Type;

        /// Format of the image data
        mut image_data_format: XI_IMG_FORMAT::Type;

        /// Selects the Test Pattern Generator Engine
        mut test_pattern_generator_selector: XI_TEST_PATTERN_GENERATOR::Type;

        /// Selects the Test Pattern to be generated by selected Generator Engine
        mut test_pattern: XI_TEST_PATTERN::Type;

        /// Camera acquisition data-rate limit on transport layer in Megabits per second.
        mut limit_bandwidth: i32;
    }
}

impl AcquisitionBuffer {
    /// Stop the image acquisition.
    ///
    /// This function consumes the acquisition buffer and returns the contained camera.
    /// All resources acquired when creating this AcquisitionBuffer using
    /// [Camera::start_acquisition()] will be freed again.
    ///
    /// When this is called, the camera will stop acquiring images and images previously acquired
    /// but not retrieved from the acquisition buffer can no longer be accessed.
    pub fn stop_acquisition(self) -> Result<Camera, XI_RETURN> {
        let err = unsafe { xiapi_sys::xiStopAcquisition(self.camera.device_handle) };
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(self.camera),
            _ => Err(err),
        }
    }

    /// Get the next image.
    ///
    /// Returns an [Image] which refers to memory in this [AcquisitionBuffer].
    /// The image will have a reference with the same lifetime as the AcquisitionBuffer making sure
    /// that it is always "safe" to use (However, it may still be overwritten in unsafe buffer mode).
    pub fn next_image<'a, T>(&'a self, timeout: Option<u32>) -> Result<Image<'a, T>, XI_RETURN> {
        let timeout = timeout.unwrap_or(u32::MAX);
        let xi_img = unsafe {
            let mut img = MaybeUninit::<XI_IMG>::zeroed().assume_init();
            img.size = size_of::<XI_IMG>() as u32;
            img
        };
        let mut image = Image::<'a, T> {
            xi_img,
            pix_type: PhantomData::default(),
        };
        unsafe {
            xiapi_sys::xiGetImage(self.camera.device_handle, timeout, &mut image.xi_img);
        }
        Ok(image)
    }
}
