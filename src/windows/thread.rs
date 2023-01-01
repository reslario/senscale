use {
    crate::{thread_id::ThreadId, windows},
    std::{io, path::PathBuf},
    winapi::um::{
        processthreadsapi::{GetCurrentThreadId, GetProcessIdOfThread, OpenThread},
        winnt::THREAD_QUERY_LIMITED_INFORMATION,
    },
};

pub fn current_id() -> ThreadId {
    unsafe { GetCurrentThreadId() }.into()
}

pub fn process_exe_path(thread_id: ThreadId) -> Option<io::Result<PathBuf>> {
    let handle = unsafe {
        OpenThread(
            THREAD_QUERY_LIMITED_INFORMATION,
            false.into(),
            thread_id.into(),
        )
    };

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
