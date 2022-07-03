use {
    super::settings::Settings,
    crate::windows::util::validate,
    std::{fs::File, io, mem::size_of, os::windows::io::AsRawHandle, ptr},
    winapi::{ctypes::c_void, um::ioapiset::DeviceIoControl},
};

pub fn write_settings(handle: &File, settings: &mut Settings) -> io::Result<()> {
    const RA_WRITE: u32 = 0x889;

    device_io_control::<_, ()>(handle, rawaccel_code(RA_WRITE), Some(settings), None)
}

fn rawaccel_code(code: u32) -> u32 {
    use winapi::um::winioctl::{CTL_CODE, FILE_ANY_ACCESS, METHOD_BUFFERED};

    const RA_DEV_TYPE: u32 = 0x8888;

    CTL_CODE(RA_DEV_TYPE, code, METHOD_BUFFERED, FILE_ANY_ACCESS)
}

fn device_io_control<I, O>(
    device: &File,
    code: u32,
    input: Option<&mut I>,
    output: Option<&mut O>,
) -> io::Result<()> {
    let input = SizedVoid::from(input);
    let output = SizedVoid::from(output);

    validate(unsafe {
        DeviceIoControl(
            device.as_raw_handle() as _,
            code,
            input.ptr,
            input.size,
            output.ptr,
            output.size,
            &mut 0,
            ptr::null_mut(),
        )
    })
}

struct SizedVoid {
    ptr: *mut c_void,
    size: u32,
}

impl<T> From<Option<&mut T>> for SizedVoid {
    fn from(val: Option<&mut T>) -> Self {
        val.map(|val| SizedVoid {
            ptr: val as *mut _ as _,
            size: size_of::<T>() as _,
        })
        .unwrap_or(SizedVoid {
            ptr: ptr::null_mut(),
            size: 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sized_void_from() {
        struct Val([u8; 8], u64);

        let val = &mut Val([0; 8], 2);
        let val_ptr = val as *mut _;
        let void = SizedVoid::from(Some(val));

        assert_eq!(void.ptr.cast(), val_ptr);
        assert_eq!(16, void.size);
    }
}
