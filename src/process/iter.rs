use {
    crate::{
        cfg::Entry,
        winutil::uninit_sized
    },
    winapi::um::{
        winnt::HANDLE,
        handleapi::CloseHandle,
        tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS}
    }
};

type ProcessName = [i8; 260];

pub struct Process {
    id: u32,
    name: ProcessName
}

impl Process {
    pub fn match_entry<'a>(&self, entries: &'a [Entry]) -> Option<Match<'a>> {
        entries
            .iter()
            .find(|entry| name_matches(&self.name, &entry.process))
            .map(|entry| Match {
                process_id: self.id,
                entry
            })
    }
}

fn name_matches(name: &ProcessName, expected: &str) -> bool {
    name.iter()
        .map(|b| *b as u8)
        .take_while(|b| *b != 0)
        .eq(expected.bytes())
}

#[derive(Copy, Clone)]
pub struct Match<'a> {
    pub process_id: u32,
    pub entry: &'a Entry
}

pub struct ProcessIter {
    snap: HANDLE,
    entry: PROCESSENTRY32,
    next: unsafe extern "system" fn(HANDLE, *mut PROCESSENTRY32) -> i32
}

impl ProcessIter {
    pub fn new() -> ProcessIter {
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        let entry = unsafe {
            uninit_sized::<PROCESSENTRY32>(|e| &mut e.dwSize)
        };

        ProcessIter {
            snap,
            entry,
            next: Process32First
        }
    }
}

impl Iterator for ProcessIter {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { (self.next)(self.snap, &mut self.entry) != 0 }
            .then(|| {
                self.next = Process32Next;

                Process {
                    id: self.entry.th32ProcessID,
                    name: self.entry.szExeFile
                }
            })
    }
}

impl Drop for ProcessIter {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.snap);
        }
    }
}
