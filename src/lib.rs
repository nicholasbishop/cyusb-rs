use std::{
    array::TryFromSliceError, convert::TryInto, fs, io, path::Path, thread,
    time::Duration,
};
pub type DeviceHandle = rusb::DeviceHandle<rusb::Context>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    /// "CY" prefix is missing
    MissingMagic,
    NotExecutable,
    AbnormalFirmware,
    InvalidChecksum,
    TruncatedData(TryFromSliceError),
    UsbError(rusb::Error),
}

struct Checksum {
    value: u32,
}

impl Checksum {
    fn new() -> Checksum {
        Checksum { value: 0 }
    }

    fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        let mut offset = 0;
        while offset < data.len() {
            let chunk = &data[offset..offset + 4];
            let val = u32::from_le_bytes(
                chunk.try_into().map_err(Error::TruncatedData)?,
            );
            self.value = self.value.overflowing_add(val).0;
            offset += 4;
        }
        Ok(())
    }
}

fn write_control(
    device: &DeviceHandle,
    address: u32,
    data: &[u8],
) -> Result<usize, Error> {
    let bytes_written = device
        .write_control(
            /*request_type=*/ 0x40,
            /*request=*/ 0xa0,
            /*value=*/ (address & 0x0000ffff) as u16,
            /*index=*/ (address >> 16) as u16,
            /*buf=*/ data,
            /*timeout=*/ Duration::from_secs(1),
        )
        .map_err(Error::UsbError)?;
    Ok(bytes_written)
}

fn control_transfer(
    device: &DeviceHandle,
    mut address: u32,
    data: &[u8],
) -> Result<(), Error> {
    let mut balance = data.len() as u32;
    let mut offset = 0;

    while balance > 0 {
        let mut b = if balance > 4096 { 4096 } else { balance };

        let bytes_written = write_control(
            device,
            address,
            &data[offset as usize..(offset + b) as usize],
        )?;

        b = bytes_written as u32;

        address += b;
        balance -= b;
        offset += b;
    }

    Ok(())
}

/// Download firmware to RAM on a Cypress FX3
pub fn program_fx3_ram(
    device: &DeviceHandle,
    path: &Path,
) -> Result<(), Error> {
    // Firmware files should be quite small, so just load the whole
    // thing in memory
    let program = fs::read(path).map_err(Error::IoError)?;

    // Program must start with "CY"
    if program[0] != b'C' || program[1] != b'Y' {
        return Err(Error::MissingMagic);
    }

    // Check that the image contains executable code
    if (program[2] & 0x01) != 0 {
        return Err(Error::NotExecutable);
    }

    // Check for a normal FW binary with checksum
    if program[3] != 0xb0 {
        return Err(Error::AbnormalFirmware);
    }

    let mut offset = 4;
    let mut checksum = Checksum::new();
    let entry_address;

    let read_u32 = |offset: &mut usize| {
        let chunk = &program[*offset..*offset + 4];
        let val =
            u32::from_le_bytes(chunk.try_into().map_err(Error::TruncatedData)?);
        *offset += 4;
        Ok(val)
    };

    // Transfer the program to the FX3
    loop {
        let length = read_u32(&mut offset)?;
        let address = read_u32(&mut offset)?;

        if length == 0 {
            entry_address = address;
            break;
        } else {
            let data = &program[offset..offset + (length as usize) * 4];
            offset += (length as usize) * 4;

            checksum.update(data)?;

            control_transfer(device, address, data)?;
        }
    }

    // Read checksum
    let expected_checksum = read_u32(&mut offset)?;
    if expected_checksum != checksum.value {
        return Err(Error::InvalidChecksum);
    }

    thread::sleep(Duration::from_secs(1));

    write_control(device, entry_address, &[])?;

    Ok(())
}
