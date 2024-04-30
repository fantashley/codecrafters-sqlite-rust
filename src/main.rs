use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::FileExt;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 100];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            let page_size = u16::from_be_bytes([header[16], header[17]]);

            println!("database page size: {}", page_size);

            let mut page_header = [0; 8];
            file.read_exact(&mut page_header)?;

            let page_type = page_header[0];

            let right_pointer = match page_type {
                0x02 | 0x05 => {
                    let mut right_pointer_bytes = [0; 4];
                    file.read_exact(&mut right_pointer_bytes)?;
                    Some(u32::from_be_bytes(right_pointer_bytes))
                }
                _ => None,
            };

            let cell_content_area = u16::from_be_bytes([page_header[5], page_header[6]]);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}

const TABLE_INTERIOR: u8 = 0x05;
const TABLE_LEAF: u8 = 0x0d;

fn parse_page(file: &mut File) -> Result<()> {
    let mut page_header = [0; 8];
    file.read_exact(&mut page_header)?;

    let page_type = page_header[0];

    let right_pointer = match page_type {
        TABLE_INTERIOR => {
            let mut right_pointer_bytes = [0; 4];
            file.read_exact(&mut right_pointer_bytes)?;
            Some(u32::from_be_bytes(right_pointer_bytes))
        }
        _ => None,
    };

    let cell_content_area = u16::from_be_bytes([page_header[5], page_header[6]]);
    let cell_content_area = if cell_content_area == 0 {
        65536
    } else {
        cell_content_area
    };
}
