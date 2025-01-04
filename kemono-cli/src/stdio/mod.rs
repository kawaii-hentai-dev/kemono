use std::{io, sync::Mutex};

use kdam::{Bar, BarExt};

pub struct WriteBar(pub Mutex<Bar>);

impl io::Write for &WriteBar {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let msg = String::from_utf8_lossy(buf);
        {
            let mut lock = self.0.lock().unwrap();
            lock.write(msg.as_ref().trim_end())?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
