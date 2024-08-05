
use windows::core::PWSTR;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Threading::{CreateProcessW, InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, DETACHED_PROCESS, EXTENDED_STARTUPINFO_PRESENT, INFINITE, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW};
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

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

    pub fn arg<T: AsRef<OsStr>>(self, arg: T) -> Self {
        self.args(&[arg])
    }

    pub fn args<T: AsRef<OsStr>>(mut self, args: &[T]) -> Self {
        self.command.extend(args.iter().map(|a| a.as_ref().into()));
        self
    }

    pub fn spawn_into(self, console: &PseudoConsole) -> anyhow::Result<ProcessHandle> {
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

        Ok(ProcessHandle(pi))
    }

    pub fn spawn_into_background(self) -> anyhow::Result<ProcessHandle> {
        let mut cmd_line = build_command(&self.command);
        dbg!(&cmd_line);
        let mut pi = PROCESS_INFORMATION::default();

        let mut si = STARTUPINFOEXW::default();
        si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;

        unsafe {
            CreateProcessW(
                None,
                PWSTR(cmd_line.as_mut_ptr()),
                None,
                None,
                false,
                EXTENDED_STARTUPINFO_PRESENT | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW,
                None,
                None,
                &si.StartupInfo,
                &mut pi
            )
        }?;

        Ok(ProcessHandle(pi))
    }
}

pub struct ProcessHandle(PROCESS_INFORMATION);

impl ProcessHandle {
    pub fn wait(&self) {
        unsafe { WaitForSingleObject(self.0.hProcess, INFINITE); }
    }
}

unsafe impl Send for ProcessHandle {}
unsafe impl Sync for ProcessHandle {}

fn build_command(command: &[OsString]) -> Vec<u16> {
    let mut cmd = Vec::new();
    for (idx, arg) in command.iter().enumerate() {
        if idx != 0 {
            cmd.push(b' ' as u16);
        }
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

pub fn current_process_path() -> PathBuf {
    let mut size: usize = 16;
    loop {
        let mut path = vec![0; size];
        let size_read = unsafe {
            GetModuleFileNameW(None, &mut path)
        } as usize;
        if size_read < size {
            path.truncate(size_read);
            return PathBuf::from(OsString::from_wide(&path));
        }
        size *= 2;
    }
}
