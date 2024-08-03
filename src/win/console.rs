
use windows::Win32::System::Console::{HPCON, ClosePseudoConsole, CreatePseudoConsole, COORD};

use super::pipe;

pub struct PseudoConsole {
    handle: HPCON,
    _in_rx: pipe::Receiver,
    out_rx: Option<pipe::Receiver>,
    in_tx: Option<pipe::Sender>,
    _out_tx: pipe::Sender,
}

impl PseudoConsole {
    pub fn new(width: i16, height: i16) -> anyhow::Result<Self> {
        let (in_tx, in_rx) = pipe::create()?;
        let (out_tx, out_rx) = pipe::create()?;
    
        let size = COORD {
            X: height,
            Y: width,
        };

        let handle = unsafe {
            CreatePseudoConsole(size, in_rx.as_handle(), out_tx.as_handle(), 0)?
        };

        Ok(Self {
            handle,
            _in_rx: in_rx,
            out_rx: Some(out_rx),
            in_tx: Some(in_tx),
            _out_tx: out_tx,
        })
    }

    pub fn handle(&self) -> HPCON {
        self.handle
    }
}

impl Drop for PseudoConsole {
    fn drop(&mut self) {
        unsafe {
            ClosePseudoConsole(self.handle);
        }
    }
}
