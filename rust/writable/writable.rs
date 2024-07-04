use std::io;

#[derive(Debug)]
pub enum Writable {
    Out(io::Stdout),
    Vec(Vec<u8>),
}

impl io::Write for Writable {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writable::Out(stdout) => stdout.write(buf),
            Writable::Vec(vec) => vec.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writable::Out(stdout) => stdout.flush(),
            Writable::Vec(_vec) => Ok(()),
        }
    }
}
