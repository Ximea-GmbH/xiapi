pub mod camera;

#[cfg(test)]
mod tests {
    use xiapi_sys::XI_RETURN;

    #[test]
    fn open_close() -> Result<(), XI_RETURN>{
        crate::camera::open_device(None)?;
        Ok(())
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
