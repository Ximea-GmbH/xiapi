/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::os::raw::c_char;
use std::str::from_utf8;

use paste::paste;
use xiapi_sys::*;

use crate::Image;
use crate::Roi;

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

            // Generate a getter for the increment
            #[doc = "Get the increment for the `" $prm "` parameter. See also [Self::" $prm "()]"]
            pub fn [<$prm _increment>](& self) -> Result<$type, XI_RETURN>{
                unsafe {self.param_increment([<XI_PRM_ $prm:upper>])}
            }

            // Generate getter for the minimum
            #[doc = "Get the minimum for the `" $prm "` parameter. See also [Self::" $prm "()]"]
            pub fn [<$prm _minimum>](& self) -> Result<$type, XI_RETURN>{
                unsafe {self.param_min([<XI_PRM_ $prm:upper>])}
            }

            // Generate getter for the maximum
            #[doc = "Get the maximum for the `" $prm "` parameter. See also [Self::" $prm "()]"]
            pub fn [<$prm _maximum>](& self) -> Result<$type, XI_RETURN>{
                unsafe {self.param_max([<XI_PRM_ $prm:upper>])}
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
pub fn open_device_manual_bandwidth(
    dev_id: Option<u32>,
    bandwidth: i32,
) -> Result<Camera, XI_RETURN> {
    let cam = unsafe {
        let bandwidth_param_c = match CStr::from_bytes_with_nul(XI_PRM_AUTO_BANDWIDTH_CALCULATION) {
            Ok(c) => c,
            Err(_) => return Err(XI_RET::XI_INVALID_ARG as XI_RETURN),
        };
        match i32::set_param(
            std::ptr::null_mut(),
            bandwidth_param_c.as_ptr(),
            XI_SWITCH::XI_OFF as i32,
        ) as XI_RET::Type
        {
            XI_RET::XI_OK => {}
            err => return Err(err as i32),
        };

        let cam = open_device(dev_id);
        match i32::set_param(
            std::ptr::null_mut(),
            bandwidth_param_c.as_ptr(),
            XI_SWITCH::XI_ON as i32,
        ) as XI_RET::Type
        {
            XI_RET::XI_OK => {}
            _ => panic!("Could not enable auto bandwidth calculation!"),
        }
        cam
    };
    match cam {
        Ok(mut cam) => {
            cam.set_limit_bandwidth(bandwidth)?;
            Ok(cam)
        }
        Err(err) => Err(err),
    }
}

/// Returns the number of available cameras.
///
/// # Examples
///
/// ```
/// # #[serial_test::file_serial]
/// # fn main() -> Result<(), xiapi::XI_RETURN>{
///     let number_devices = xiapi::number_devices()?;
///     let mut cameras = Vec::with_capacity(number_devices as usize);
///     for i in 0..number_devices {
///         cameras.push(xiapi::open_device(Some(i))?);
///     }
/// # Ok(())
/// # }
pub fn number_devices() -> Result<u32, XI_RETURN> {
    unsafe {
        let mut value = 0u32;
        let res = xiapi_sys::xiGetNumberDevices(&mut value);
        match res as XI_RET::Type {
            XI_RET::XI_OK => Ok(value),
            _ => Err(res),
        }
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

    unsafe fn param_increment<T: ParamType>(&self, param: &'static [u8]) -> Result<T, XI_RETURN> {
        self.param_info(param, XI_PRM_INFO_INCREMENT)
    }

    unsafe fn param_min<T: ParamType>(&self, param: &'static [u8]) -> Result<T, XI_RETURN> {
        self.param_info(param, XI_PRM_INFO_MIN)
    }

    unsafe fn param_max<T: ParamType>(&self, param: &'static [u8]) -> Result<T, XI_RETURN> {
        self.param_info(param, XI_PRM_INFO_MAX)
    }

    unsafe fn param_info<T: ParamType>(
        &self,
        param: &'static [u8],
        info_modifier: &'static [u8],
    ) -> Result<T, XI_RETURN> {
        let modified_param = param_suffix(param, info_modifier)?;
        self.param(modified_param.as_bytes())
    }

    /// Set the region of interest on this camera.
    ///
    /// Return the region of interest that was actually set to the camera.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[serial_test::file_serial]
    /// # fn main() -> Result<(), xiapi_sys::XI_RETURN> {
    ///     let mut cam = xiapi::open_device(None)?;
    ///     let roi = xiapi::Roi{
    ///         offset_x: 100,
    ///         offset_y: 100,
    ///         width: 100,
    ///         height: 100
    ///     };
    ///     let actual_roi = cam.set_roi(&roi)?;
    /// # Ok(())
    /// # }
    ///
    pub fn set_roi(&mut self, roi: &Roi) -> Result<Roi, XI_RETURN> {
        self.set_offset_x(0)?;
        self.set_offset_y(0)?;

        let width_inc = self.width_increment()?;
        let width = roi.width - (roi.width % width_inc);
        self.set_width(width)?;

        let height_inc = self.height_increment()?;
        let height = roi.height - (roi.height % height_inc);
        self.set_height(height)?;

        let offset_x_inc = self.offset_x_increment()?;
        let offset_x = roi.offset_x - (roi.offset_x % offset_x_inc);
        self.set_offset_x(offset_x)?;

        let offset_y_inc = self.offset_y_increment()?;
        let offset_y = roi.offset_y - (roi.offset_y % offset_y_inc);
        self.set_offset_y(offset_y)?;

        let actual_roi = Roi {
            offset_x,
            offset_y,
            width,
            height,
        };
        Ok(actual_roi)
    }

    /// Returns the current roi from this camera
    pub fn roi(&self) -> Result<Roi, XI_RETURN> {
        let width = self.width()?;
        let height = self.height()?;
        let offset_x = self.offset_x()?;
        let offset_y = self.offset_y()?;
        let result = Roi {
            offset_x,
            offset_y,
            width,
            height,
        };
        Ok(result)
    }

    /// Convenience method to read counters from the camera with a single call
    /// See also [Self.counter_selector] and [Self.counter_value]
    pub fn counter(
        &mut self,
        counter_selector: XI_COUNTER_SELECTOR::Type,
    ) -> Result<i32, XI_RETURN> {
        let prev_selector = self.counter_selector()?;
        self.set_counter_selector(counter_selector)?;
        let result = self.counter_value()?;
        self.set_counter_selector(prev_selector)?;
        Ok(result)
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

        /// Immage ROI height (number of lines)
        mut height: u32;

        /// Image ROI width (number of columns)
        mut width: u32;

        /// Image ROI offset in the horizontal direction
        mut offset_x: u32;

        /// Image ROI offset in the vertical direction
        mut offset_y: u32;

        /// Camera acquisition data-rate limit on transport layer in Megabits per second.
        mut limit_bandwidth: i32;

        /// Available interface bandwidth measured by automatic bandwidth measurement.
        available_bandwidth: i32;

        /// Defines the source of trigger
        mut trg_source: XI_TRG_SOURCE::Type;

        /// Selects the type of trigger
        mut trg_selector: XI_TRG_SELECTOR::Type;

        /// Selects the type of trigger overlap
        mut trg_overlap: XI_TRG_OVERLAP::Type;

        /// Sets the number of frames to be triggered for each trigger signal.
        /// This setting is only valid if the trigger selector is set to XI_TRG_SEL_FRAME_BURST_START
        mut acq_frame_burst_count: u32;

        /// Selects a GPI
        mut gpi_selector: XI_GPI_SELECTOR::Type;

        /// Defines functionality for the selected GPI
        mut gpi_mode: XI_GPI_MODE::Type;

        /// Selects a GPO
        mut gpo_selector: XI_GPO_SELECTOR::Type;

        /// Defines functionality for the selected GPO
        mut gpo_mode: XI_GPO_MODE::Type;

        /// Selects a LED
        mut led_selector: XI_LED_SELECTOR::Type;

        /// Defines functionality for the selected LED
        mut led_mode: XI_LED_MODE::Type;

        /// Enable or disable signal debounce for selected GPI
        mut debounce_en: XI_SWITCH::Type;

        /// Set user data to be stored in the image header
        mut image_user_data: u32;

        /// Set the bit depth for the ADCs on the sensor
        /// # Examples
        /// ```
        /// # #[serial_test::file_serial()]
        /// # fn main() -> Result<(), xiapi::XI_RETURN>{
        /// # use xiapi_sys::XI_IMG_FORMAT::XI_RAW16;
        /// # use xiapi::XI_BIT_DEPTH::XI_BPP_12;
        /// let mut cam = xiapi::open_device(None)?;
        /// cam.set_image_data_format(XI_RAW16)?;
        /// cam.set_sensor_data_bit_depth(XI_BPP_12)?;
        /// cam.set_output_data_bit_depth(XI_BPP_12)?;
        /// cam.set_image_data_bit_depth(XI_BPP_12)?;
        /// # assert_eq!(cam.sensor_data_bit_depth()?, XI_BPP_12);
        /// # assert_eq!(cam.output_data_bit_depth()?, XI_BPP_12);
        /// # assert_eq!(cam.image_data_bit_depth()?, XI_BPP_12);
        /// # Ok(())
        /// }
        mut sensor_data_bit_depth: XI_BIT_DEPTH::Type;

        /// Set the bit depth send from the camera to the PC
        mut output_data_bit_depth: XI_BIT_DEPTH::Type;

        /// Bit depth of the image returned by [Self::next_image()]
        mut image_data_bit_depth: XI_BIT_DEPTH::Type;

        /// Enable column fpn correction in camera
        mut column_fpn_correction: XI_SWITCH::Type;

        /// Enable row fpn correction in camera
        mut row_fpn_correction: XI_SWITCH::Type;

        /// Enable column black offset correction
        mut column_black_offset_correction: XI_SWITCH::Type;

        /// Enable row black offset correction
        mut row_black_offset_correction: XI_SWITCH::Type;

        /// Select the frame counter to read
        mut counter_selector: XI_COUNTER_SELECTOR::Type;

        /// Read the value of a frame counter selected with [Self::set_counter_selector]
        counter_value: i32;

        /// Select a sensor specific feature
        mut sensor_feature_selector: XI_SENSOR_FEATURE_SELECTOR::Type;

        /// Set a value for the feature selected with [Self::set_sensor_feature_selector]
        mut sensor_feature_value: i32;

        /// Read the sensor clock frequency in Hz
        sensor_clock_freq_hz: f32;

        /// Data move policy
        mut buffer_policy: i32;

        /// Auto white balance mode.
        mut auto_wb: i32;

        /// White balance Red coefficient.
        mut wb_kr: f32;

        /// White balance Green coefficient.
        mut wb_kg: f32;

        /// White balance Blue coefficient.
        mut wb_kb: f32;
    }
}

impl Deref for Camera {
    type Target = HANDLE;

    /// Returns a reference to the wrapped device handle.
    ///
    /// While getting the handle itself is safe, everything that can practically be done with it
    /// should be considered unsafe. Especially operations that change the state of the camera
    /// (e.g. setting parameters) are undefined behavior.
    fn deref(&self) -> &Self::Target {
        &self.device_handle
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
        let ret_code = unsafe {
            xiapi_sys::xiGetImage(self.camera.device_handle, timeout, &mut image.xi_img)
        };
        match ret_code as u32 {
            xiapi_sys::XI_RET::XI_OK => Ok(image),
            _ => Err(ret_code),
        }
    }

    /// Send a software trigger signal to the camera.
    ///
    /// Trigger source has to be set to XI_TRG_SOFTWARE for this to take effect
    ///
    /// # Examples
    /// ```
    /// # #[serial_test::file_serial]
    /// # fn main() -> Result<(), xiapi_sys::XI_RETURN> {
    ///     let mut cam = xiapi::open_device(None)?;
    ///     cam.set_trg_source(xiapi_sys::XI_TRG_SOURCE::XI_TRG_SOFTWARE)?;
    ///     let mut acq_buffer = cam.start_acquisition()?;
    ///     acq_buffer.software_trigger()?;
    ///     let img = acq_buffer.next_image::<u8>(None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn software_trigger(&mut self) -> Result<(), XI_RETURN> {
        unsafe { self.camera.set_param(XI_PRM_TRG_SOFTWARE, XI_SWITCH::XI_ON) }
    }

    /// Set the exposure time of the camera while streaming.
    pub fn set_exposure(&mut self, exposure: f32) -> Result<(), XI_RETURN> {
        let err = unsafe { xiapi_sys::xiSetParamInt(self.camera.device_handle, b"exposure: direct_update\0".as_ptr().cast(), exposure as i32) };
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(()),
            _ => Err(err),
        }
    }

    /// Set the gain of the camera while streaming.
    pub fn set_gain(&mut self, gain: f32) -> Result<(), XI_RETURN> {
        let param_name = unsafe{ param_suffix(XI_PRM_GAIN, XI_PRMM_DIRECT_UPDATE).unwrap()};
        let param_c = CStr::from_bytes_with_nul( param_name.as_bytes() ).unwrap();
        let err = unsafe { xiapi_sys::xiSetParamFloat(self.camera.device_handle, param_c.as_ptr(), gain) };
        match err as XI_RET::Type {
            XI_RET::XI_OK => Ok(()),
            _ => Err(err),
        }
    }

}

//=================================================================================
unsafe fn param_suffix(param: &[u8], info_modifier: &[u8]) -> Result<String, XI_RETURN> {
    // Strings need to be sanitized and then concatenated
    let param_utf8 = from_utf8(param).or(Err(XI_RET::XI_INVALID_ARG as i32))?;
    let modifier_utf8 =
        from_utf8(info_modifier).expect("UTF8 error on API constant -> Unreachable");
    // We have to specifically trim the null character from the first string
    let modified_param = format!(
        "{}{}",
        param_utf8.trim_matches(char::from(0)),
        modifier_utf8
    );
    Ok(modified_param)
}
