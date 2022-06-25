use {
    super::util::validate,
    std::{
        io,
        ffi::OsString,
        path::PathBuf,
        os::windows::prelude::OsStringExt
    },
    winapi::{
        shared::minwindef::MAX_PATH,
        um::{
            handleapi::CloseHandle,
            processthreadsapi::OpenProcess,
            winbase::QueryFullProcessImageNameW,
            winnt::PROCESS_QUERY_LIMITED_INFORMATION
        }
    }
};

pub fn exe_path(id: u32) -> io::Result<PathBuf> {
    let proc = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false.into(), id) };

    let mut buf = [0; MAX_PATH];
    let mut end = MAX_PATH as _;
    validate(unsafe { QueryFullProcessImageNameW(proc, 0, buf.as_mut_ptr(), &mut end) })?;
    unsafe { CloseHandle(proc); }

    Ok(OsString::from_wide(&buf[..end as usize]).into())
}
