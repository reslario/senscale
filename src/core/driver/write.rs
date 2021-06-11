use {
    super::settings::Settings,
    winapi::{
        ctypes::c_void,
        um::ioapiset::DeviceIoControl
    },
    std::{
        io,
        ptr,
        fs::File,
        mem::size_of,
        os::windows::io::AsRawHandle
    },
};

pub fn write_settings(settings: &mut Settings) -> io::Result<()> {
    const RA_WRITE: u32 = 0x889;

    rawaccel_control::<_, ()>(rawaccel_code(RA_WRITE), Some(settings), None)
}

fn rawaccel_code(code: u32) -> u32 {
    use winapi::um::winioctl::{CTL_CODE, METHOD_BUFFERED, FILE_ANY_ACCESS};

    const RA_DEV_TYPE: u32 = 0x8888;

    CTL_CODE(RA_DEV_TYPE, code, METHOD_BUFFERED, FILE_ANY_ACCESS)
}

fn rawaccel_control<I, O>(code: u32, input: Option<&mut I>, output: Option<&mut O>) -> io::Result<()> {
    File::open(r"\\.\rawaccel")
        .and_then(|device| device_io_control(&device, code, input, output))
}

fn device_io_control<I, O>(device: &File, code: u32, input: Option<&mut I>, output: Option<&mut O>) -> io::Result<()> {
    let input = SizedVoid::from(input);
    let output = SizedVoid::from(output);

    let success = unsafe {
        DeviceIoControl(
            device.as_raw_handle() as _,
            code,
            input.ptr,
            input.size,
            output.ptr,
            output.size,
            &mut 0,
            ptr::null_mut()
        )
    } != 0;

    if success {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

struct SizedVoid {
    ptr: *mut c_void,
    size: u32
}

impl <T> From<Option<&mut T>> for SizedVoid {
    fn from(val: Option<&mut T>) -> Self {
        match val {
            Some(val) => SizedVoid {
                ptr: val as *mut _ as _,
                size: size_of::<T>() as _
            },
            None => SizedVoid {
                ptr: ptr::null_mut(),
                size: 0
            }
        }
    }
}
