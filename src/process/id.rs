use {
    std::mem::size_of,
    winapi::um::{
        winnt::HANDLE,
        tlhelp32::{CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS}
    }
};

type ProcessName = [i8; 260];

pub struct ProcessIds<'a> {
    name: &'a str,
    snap: HANDLE,
    entry: PROCESSENTRY32,
    next: unsafe extern "system" fn(HANDLE, *mut PROCESSENTRY32) -> i32
}

impl <'a> ProcessIds<'a> {
    pub fn for_name(name: &'a str) -> ProcessIds<'a> {
        let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        let entry = PROCESSENTRY32 {
            dwSize: size_of::<PROCESSENTRY32>() as _,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260],
        };

        ProcessIds {
            name,
            snap,
            entry,
            next: Process32First
        }
    }
}

impl <'a> Iterator for ProcessIds<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        while unsafe { (self.next)(self.snap, &mut self.entry) != 0 } {
            self.next = Process32Next;

            if name_matches(&self.entry.szExeFile, self.name) {
                return self.entry.th32ProcessID.into()
            }
        }

        None
    }
}

fn name_matches(name: &ProcessName, expected: &str) -> bool {
    name.iter()
        .map(|b| *b as u8)
        .take_while(|b| *b != 0)
        .eq(expected.bytes())
}
