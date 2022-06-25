use std::io;

pub fn validate(result: i32) -> io::Result<()> {
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[allow(clippy::uninit_assumed_init)]
pub unsafe fn uninit_sized<T>(size_member: fn(&mut T) -> &mut u32) -> T {
    use std::mem::{MaybeUninit, size_of};

    let mut info = MaybeUninit::<T>::uninit().assume_init();
    *size_member(&mut info) = size_of::<T>() as _;
    info
}
