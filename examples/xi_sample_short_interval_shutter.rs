/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */
use image::{ImageBuffer, Luma};
use xiapi_sys::XI_SENSOR_FEATURE_SELECTOR::XI_SENSOR_FEATURE_SHORT_INTERVAL_SHUTTER;
use xiapi_sys::XI_TRG_SOURCE::XI_TRG_SOFTWARE;

fn main() -> Result<(), i32> {
    // Set a manual bandwidth just to make sure sensor clocks are always the same
    let mut cam = xiapi::open_device_manual_bandwidth(Some(1), 2500)?;

    // Select and enable the short interval shutter feature (available only on certain camera models)
    cam.set_sensor_feature_selector(XI_SENSOR_FEATURE_SHORT_INTERVAL_SHUTTER)?;
    cam.set_sensor_feature_value(1)?;

    // Set up the trigger source
    cam.set_trg_source(XI_TRG_SOFTWARE)?;
    let mut buffer = cam.start_acquisition()?;

    // Send a single trigger signal
    buffer.software_trigger()?;

    // Read out two frames
    for i in 0..2 {
        let image = buffer.next_image::<u8>(None)?;
        let image_buffer = ImageBuffer::<Luma<u8>, _>::from(image);
        image_buffer
            .save(format!("short_interval_shutter_test{i}.png"))
            .unwrap();
    }
    Ok(())
}
