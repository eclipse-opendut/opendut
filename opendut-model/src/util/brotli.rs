use std::io;
use std::io::{Read, Write};

pub fn compress(buffer: &mut Vec<u8>, content: &[u8]) -> io::Result<usize> {
    brotli::CompressorReader::new(content, 4096, 11, 20)
        .read_to_end(buffer)
}

pub fn decompress(buffer: &mut Vec<u8>, compressed: &[u8]) -> io::Result<()> {
    brotli::DecompressorWriter::new(buffer, 4096)
        .write_all(compressed)
}
