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
            _ => Err("invalid target".to_string())
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "fx3", about = "Write firmware to a Cypress FX3 device.")]
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
}
