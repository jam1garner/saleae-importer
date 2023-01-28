//! A library for reading and writing Saleae Logic 2 binary capture data
use std::{io::BufWriter, path::Path};

use binrw::{
    io::{self, BufReader},
    prelude::*,
};

/// A binary representing data parsed from a Saleae Logic 2 digital.bin export file
#[binrw]
#[brw(little, magic = b"<SALEAE>")]
#[derive(Debug)]
pub struct SaleaeExport {
    /// Version of the export file format, only version 0 is supported
    #[br(assert(version == 0))]
    pub version: i32,

    /// The underlying data of the file
    pub file_data: Data,
}

/// The underlying data of the file, either digital or analog
#[binrw]
#[derive(Debug)]
pub enum Data {
    #[brw(magic = 0u32)]
    Digital(DigitalData),

    #[brw(magic = 1u32)]
    Analog(AnalogData),
}

/// The data for digital captures exported by saleae logic 2
#[binrw]
#[derive(Debug)]
pub struct DigitalData {
    /// The initial state of the
    pub initial_state: u32,
    pub begin_time: f64,
    pub end_time: f64,

    #[bw(calc = transition_times.len() as u64)]
    num_transitions: u64,

    #[br(count = num_transitions)]
    pub transition_times: Vec<f64>,
}

/// The data for analog capture exports
#[binrw]
#[derive(Debug)]
pub struct AnalogData {
    /// The time the capture begins
    pub begin_time: f64,

    /// The sample rate (in hertz)
    pub sample_rate: u64,

    /// The level of downsampling being done on the capture
    pub downsample: u64,

    #[bw(calc = samples.len() as u64)]
    pub num_samples: u64,

    /// The voltage within the given sample
    #[br(count = num_samples)]
    pub samples: Vec<f64>,
}

impl SaleaeExport {
    /// Read an export from disk
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        Self::read(BufReader::new(std::fs::File::open(path)?))
    }

    /// Read export data from a reader
    pub fn read(mut reader: impl io::Read + io::Seek) -> io::Result<Self> {
        reader.read_le().map_err(|err| {
            if let binrw::Error::Io(io_err) = err {
                io_err
            } else {
                io::Error::new(io::ErrorKind::Other, err)
            }
        })
    }

    /// Parse export data from a buffer
    pub fn read_from_bytes(bytes: &[u8]) -> io::Result<Self> {
        Self::read(binrw::io::Cursor::new(bytes))
    }

    /// Save data back to a file
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();

        self.write_to(BufWriter::new(std::fs::File::create(path)?))
    }

    /// Write data back to a writer
    pub fn write_to(&self, mut writer: impl io::Write + io::Seek) -> io::Result<()> {
        writer.write_le(self).map_err(|err| {
            if let binrw::Error::Io(io_err) = err {
                io_err
            } else {
                io::Error::new(io::ErrorKind::Other, err)
            }
        })
    }
}
