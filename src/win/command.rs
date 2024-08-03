
use windows::core::PWSTR;
use windows::Win32::System::Threading::{CreateProcessW, InitializeProcThreadAttributeList, UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW};
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;

use super::console::PseudoConsole;
use super::mem;

pub struct Command {
    command: Vec<OsString>,
}

impl Command {
    pub fn new(command: impl AsRef<OsStr>) -> Self {
        Self {
            command: vec![command.as_ref().into()],
        }
    }

    pub fn args<T: AsRef<OsStr>>(mut self, args: Vec<T>) -> Self {
        self.command.extend(args.into_iter().map(|a| a.as_ref().into()));
        self
    }

    pub fn spawn_into(self, console: &PseudoConsole) -> anyhow::Result<PROCESS_INFORMATION> {
        let cmd_line_osstr = self.command.join(OsStr::new("\0"));
        let cmd_line = cmd_line_osstr.encode_wide();
        
        // Put command line into explicit heap memory because we will hand
        // it off to CreateProcess.
        let cmd_line_heap = mem::HeapMemory::alloc(cmd_line.clone().count()*std::mem::size_of::<u16>());
        if cmd_line_heap.is_invalid() {
            return Err(anyhow::anyhow!("allocation failed"));
        }
        unsafe {
            let ptr = cmd_line_heap.0 as *mut u16;
            for (idx, c) in cmd_line.enumerate() {
                *ptr.add(idx) = c;
            }
        }

        let si = prepare_startup_information(console)?;

        let mut pi = PROCESS_INFORMATION::default();
        unsafe {
            CreateProcessW(
                None,
                PWSTR(cmd_line_heap.into_ptr() as *mut u16),
                None,
                None,
                false,
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                None,
                &si.StartupInfo,
                &mut pi
            )
        }?;

        Ok(pi)
    }
}

fn prepare_startup_information(console: &PseudoConsole) -> anyhow::Result<STARTUPINFOEXW> {
    let mut si = STARTUPINFOEXW::default();
    si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

    // Discover the size required for the list
    let mut size: usize = 0;
    unsafe {
        let _ = InitializeProcThreadAttributeList(LPPROC_THREAD_ATTRIBUTE_LIST(std::ptr::null_mut()), 1, 0, &mut size);
    }

    // Allocate memory
    let attr_list = mem::HeapMemory::alloc(size);
    if attr_list.is_invalid() {
        return Err(anyhow::anyhow!("Allocation failed"));
    }
    si.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list.0);

    unsafe {
        InitializeProcThreadAttributeList(si.lpAttributeList, 1, 0, &mut size)
    }?;

    let console_handle = console.handle();
    unsafe {
        UpdateProcThreadAttribute(
            si.lpAttributeList,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
            Some(console_handle.0 as *const std::ffi::c_void),
            std::mem::size_of_val(&console_handle),
            None,
            None
        )
    }?;

    // forget pointer
    attr_list.into_ptr();

    Ok(si)
}
