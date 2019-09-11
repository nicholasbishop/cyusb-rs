use rusb::UsbContext;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
enum Target {
    Ram,
    I2c,
    Spi,
}

impl std::str::FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: &str = &s.to_ascii_lowercase();
        match s {
            "ram" => Ok(Target::Ram),
            "i2c" => Ok(Target::I2c),
            "spi" => Ok(Target::Spi),
            _ => Err("invalid target".to_string()),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "cyusb_programmer",
    about = "Write firmware to a Cypress FX3 device."
)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    image: PathBuf,

    /// Select between multiple devices
    #[structopt(short, long, default_value = "0")]
    index: usize,

    /// RAM, I2C, or SPI
    #[structopt(short, long, default_value = "RAM")]
    target: Target,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let mut context = rusb::Context::new().unwrap();

    for mut device in context.devices().unwrap().iter() {
        let desc = device.device_descriptor().unwrap();

        if desc.vendor_id() != 0x04b4 || desc.product_id() != 0x00f3 {
            continue;
        }

        println!(
            "Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            desc.vendor_id(),
            desc.product_id()
        );
    }
}
