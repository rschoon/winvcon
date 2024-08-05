
use std::io::{Read, Write};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::Storage::FileSystem::{FlushFileBuffers, ReadFile, WriteFile};

pub struct Receiver(HANDLE);

impl Receiver {
    pub fn as_handle(&self) -> HANDLE {
        self.0
    }
}

unsafe impl Sync for Receiver {}
unsafe impl Send for Receiver {}

impl Read for &Receiver {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = buf.len() as u32;
        unsafe { ReadFile(self.0, Some(buf), Some(&mut read), None) }?;
        Ok(read as usize)
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub struct Sender(HANDLE);

impl Sender {
    pub fn as_handle(&self) -> HANDLE {
        self.0
    }
}

impl Write for &Sender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut write = buf.len() as u32;
        unsafe { WriteFile(self.0, Some(buf), Some(&mut write), None) }?;
        Ok(write as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unsafe { FlushFileBuffers(self.0) }?;
        Ok(())
    }
}

unsafe impl Sync for Sender {}
unsafe impl Send for Sender {}

impl Drop for Sender {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn create() -> anyhow::Result<(Sender, Receiver)> {
    let mut rx: HANDLE = HANDLE::default();
    let mut tx: HANDLE = HANDLE::default();

    unsafe {
        CreatePipe(&mut rx, &mut tx, None, 0)
    }?;

    Ok((Sender(tx), Receiver(rx)))
}
