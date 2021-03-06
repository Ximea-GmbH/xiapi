/*
 * Copyright (c) 2022. XIMEA GmbH - All Rights Reserved
 */
fn main() -> Result<(), i32> {
    let mut cam = xiapi::open_device(None)?;

    cam.set_exposure(10000.0)?;

    let buffer = cam.start_acquisition()?;

    for i in 0..10 {
        let image = buffer.next_image::<u8>(None)?;
        let pixel = image.pixel(0, 0);
        match pixel {
            Some(&pixel_value) => println!(
                "Image {} ({}x{}) received from camera. First pixel value: {}",
                i,
                image.width(),
                image.height(),
                pixel_value
            ),
            None => unreachable!("Could not get pixel value from image!"),
        }
    }
    Ok(())
}
