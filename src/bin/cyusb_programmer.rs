use log::{error, info};
use rusb::UsbContext;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, PartialEq)]
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
    env_logger::builder()
        .format_target(false)
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    let opt = Opt::from_args();
    if opt.target != Target::Ram {
        error!("only the RAM target works currently");
        exit(1);
    }

    let context = rusb::Context::new().unwrap();

    let devices = context
        .devices()
        .unwrap()
        .iter()
        .filter(|d| {
            let desc = d.device_descriptor().unwrap();
            desc.vendor_id() == 0x04b4 && desc.product_id() == 0x00f3
        })
        .collect::<Vec<_>>();

    if devices.len() == 1 {
        info!("1 device detected");
    } else {
        info!("{} devices detected", devices.len());
    }

    if let Some(device) = devices.get(opt.index) {
        if let Err(err) =
            cyusb::program_fx3_ram(&device.open().unwrap(), &opt.image)
        {
            error!("program_fx3_ram failed: {:?}", err);
            exit(1);
        }
    } else {
        error!("invalid index, detected {} device(s)", devices.len());
        exit(1);
    }
}
