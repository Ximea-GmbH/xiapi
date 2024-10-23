/*
 * Copyright (c) 2024. XIMEA GmbH - All Rights Reserved
 */
use xiapi_sys::XI_IMG_FORMAT;
use image::{ImageBuffer, Rgb};
fn main() -> Result<(), i32> {
    let mut cam = xiapi::open_device(None)?; // Open the camera device
    cam.set_exposure(10000.0)?; // Set the exposure time to 10000 microseconds
    cam.set_image_data_format(XI_IMG_FORMAT::XI_RGB24)?; // Set the image format to RGB24

    let buffer = cam.start_acquisition()?; // Start the image acquisition process
    let image = buffer.next_image::<u8>(None)?; // Get the next image from the buffer
    let image_buffer = ImageBuffer::<Rgb<u8>, _>::from(image); // Convert the image to an ImageBuffer
    image_buffer.save("example.jpg").expect("Could not save image!"); // Save the image to a file

    Ok(())
}