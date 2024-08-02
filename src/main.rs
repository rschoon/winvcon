use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Console::{ClosePseudoConsole, CreatePseudoConsole, COORD};
use windows::Win32::System::Pipes::CreatePipe;

fn main() {
    let size: COORD = COORD { X: 80, Y: 24 };
    let mut in_rx: HANDLE = HANDLE::default();
    let mut out_tx: HANDLE = HANDLE::default();
    let mut in_tx: HANDLE = HANDLE::default();
    let mut out_rx: HANDLE = HANDLE::default();
    
    unsafe {
        CreatePipe(&mut in_rx, &mut in_tx, None, 0).unwrap();
    }

    unsafe {
        CreatePipe(&mut out_rx, &mut out_tx, None, 0).unwrap();
    }

    let console = unsafe {
        CreatePseudoConsole(size, in_rx, out_rx, 0)
    }.unwrap();

    dbg!(&console);

    unsafe { ClosePseudoConsole(console); }
}
