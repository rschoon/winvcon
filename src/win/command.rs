
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
        let mut cmd_line = build_command(&self.command);

        let si = prepare_startup_information(console)?;
        let mut pi = PROCESS_INFORMATION::default();
        unsafe {
            CreateProcessW(
                None,
                PWSTR(cmd_line.as_mut_ptr()),
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

        // Free the attribute list
        unsafe {
            mem::HeapMemory::from_ptr(si.lpAttributeList.0);
        }

        Ok(pi)
    }
}

fn build_command(command: &[OsString]) -> Vec<u16> {
    let mut cmd = Vec::new();
    cmd.push(b'"' as u16);
    cmd.extend(command[0].encode_wide());
    cmd.push(b'"' as u16);

    for arg in &command[1..] {
        cmd.push(b' ' as u16);
        cmd.push(b'\"' as u16);
        for c in arg.encode_wide() {
            if c == '\\' as u16 {
                cmd.extend(['\\' as u16, '\\' as u16]);
            } else {
                if c == '\"' as u16 {
                    cmd.push('\\' as u16);
                }
                cmd.push(c);
            }
        }
        cmd.push(b'\"' as u16);
    }

    cmd.push(0);

    cmd
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
    let attr_list = mem::HeapMemory::alloc(size)?;
    si.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list.as_ptr());

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
