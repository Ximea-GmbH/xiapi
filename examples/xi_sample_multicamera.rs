use xiapi::number_devices;
use xiapi::open_device;
use xiapi::XI_RETURN;
use xiapi::XI_TRG_SOURCE::XI_TRG_SOFTWARE;

fn main() -> Result<(), XI_RETURN> {
    let num_devs = number_devices()?;
    let mut acq_buffers = Vec::with_capacity(num_devs as usize);
    for i in 0..num_devs {
        let mut cam = open_device(Some(i))?;
        cam.set_exposure(1000 as f32)?;
        cam.set_trg_source(XI_TRG_SOFTWARE)?;
        acq_buffers.push(cam.start_acquisition()?);
    }
    for buf in &mut acq_buffers {
        buf.software_trigger()?;
    }
    for buf in &acq_buffers {
        let img = buf.next_image::<u8>(None)?;
        println!(
            "Received image! Width: {}, Height: {}",
            img.width(),
            img.height()
        );
    }
    Ok(())
}
