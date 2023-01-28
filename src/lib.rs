//! A library for reading and writing Saleae Logic 2 binary capture data
//!
//! ### Example
//!
//! ```
//! use saleae_importer::SaleaeExport;
//!
//! let data = SaleaeExport::open("digital_0.bin").unwrap();
//!
//! for (is_high, time_len) in data.assume_digital().iter_samples() {
//!     println!("bit state: {is_high} | time: {time_len}");
//! }
//! ```
use std::{io::BufWriter, path::Path};

use binrw::{
    io::{self, BufReader},
    prelude::*,
};

/// A binary representing data parsed from a Saleae Logic 2 digital.bin export file
#[binrw]
#[brw(little, magic = b"<SALEAE>")]
#[derive(Debug, Clone)]
pub struct SaleaeExport {
    /// Version of the export file format, only version 0 is supported
    #[br(assert(version == 0))]
    pub version: i32,

    /// The underlying data of the file
    pub file_data: Data,
}

/// The underlying data of the file, either digital or analog
#[binrw]
#[derive(Debug, Clone)]
pub enum Data {
    #[brw(magic = 0u32)]
    Digital(DigitalData),

    #[brw(magic = 1u32)]
    Analog(AnalogData),
}

/// The data for digital captures exported by saleae logic 2
#[binrw]
#[derive(Debug, Clone)]
pub struct DigitalData {
    /// The initial state of the capture
    pub initial_state: State,
    pub begin_time: f64,
    pub end_time: f64,

    #[bw(calc = transition_times.len() as u64)]
    num_transitions: u64,

    #[br(count = num_transitions)]
    pub transition_times: Vec<f64>,
}

/// An iterator over the sample data returning pairs of (bool, f64) representing
/// whether the sample is high (true) or low (false) as well as the length of the sample
pub struct SampleIter<'samples> {
    samples: &'samples [f64],
    current: usize,
    initial: bool,
}

impl<'samples> Iterator for SampleIter<'samples> {
    type Item = (bool, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let ret = (
            // alternating signal
            self.initial ^ ((self.current & 1) != 0),
            // this sample time minus last sample time
            *self.samples.get(self.current)?
                - self
                    .current
                    .checked_sub(1)
                    .and_then(|last| self.samples.get(last).copied())
                    .unwrap_or(0.0),
        );

        self.current += 1;

        Some(ret)
    }
}

impl DigitalData {
    pub fn iter_samples(&self) -> SampleIter<'_> {
        SampleIter {
            samples: &self.transition_times,
            current: 0,
            initial: self.initial_state.into(),
        }
    }
}

/// The state of whether a sample is high or low
#[binwrite]
#[derive(Debug, Copy, Clone)]
pub enum State {
    #[bw(magic = 0u32)]
    Low = 0,

    #[bw(magic = 1u32)]
    High = 1,
}

impl BinRead for State {
    type Args = ();

    fn read_options<R: io::Read + io::Seek>(
        reader: &mut R,
        _options: &binrw::ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        reader
            .read_le()
            .map(|x: u32| if x == 0 { State::Low } else { State::High })
    }
}

impl From<State> for bool {
    fn from(value: State) -> Self {
        matches!(value, State::High)
    }
}

/// The data for analog capture exports
#[binrw]
#[derive(Debug, Clone)]
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
    /// Assume the underlying data is analog and proceed accordingly
    ///
    /// ## Panics
    ///
    /// Panics if the capture being processed is digital
    pub fn assume_analog(self) -> AnalogData {
        match self.file_data {
            Data::Analog(analog) => analog,
            Data::Digital(_) => panic!("Expected data to be digital, found analog"),
        }
    }

    /// Assume the underlying data is digital and proceed accordingly
    ///
    /// ## Panics
    ///
    /// Panics if the capture being processed is analog
    pub fn assume_digital(self) -> DigitalData {
        match self.file_data {
            Data::Digital(digital) => digital,
            Data::Analog(_) => panic!("Expected data to be digital, found analog"),
        }
    }

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
