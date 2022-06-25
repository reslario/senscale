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

    let mut value = MaybeUninit::<T>::uninit().assume_init();
    *size_member(&mut value) = size_of::<T>() as _;
    value
}

#[cfg(test)]
mod test {
    #[test]
    fn validate() {
        assert!(super::validate(0).is_err());
        assert!(super::validate(!0).is_ok())
    }

    #[test]
    fn uninit_sized() {
        struct Test {
            size: u32,
            #[allow(unused)]
            whatever: u64
        }

        assert_eq!(
            unsafe { super::uninit_sized::<Test>(|test| &mut test.size ) }.size,
            std::mem::size_of::<Test>() as u32
        )
    }
}
