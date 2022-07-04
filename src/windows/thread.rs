use {
    crate::windows,
    std::{io, path::PathBuf},
    winapi::um::{
        processthreadsapi::{GetCurrentThreadId, GetProcessIdOfThread, OpenThread},
        winnt::THREAD_QUERY_LIMITED_INFORMATION,
    },
};

pub fn current_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub fn process_exe_path(thread_id: u32) -> Option<io::Result<PathBuf>> {
    let handle = unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, false.into(), thread_id) };

    if handle.is_null() {
        return None
    }

    let process_id = unsafe { GetProcessIdOfThread(handle) };
    windows::process::exe_path(process_id).into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn current_process_exe() {
        assert_eq!(
            std::env::current_exe().unwrap(),
            process_exe_path(current_id()).unwrap().unwrap()
        )
    }
}
