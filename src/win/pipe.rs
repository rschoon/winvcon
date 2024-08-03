
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Pipes::CreatePipe;

pub struct Receiver(HANDLE);

impl Receiver {
    pub fn as_handle(&self) -> HANDLE {
        self.0
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
