# cyusb

Host crate for interacting with Cypress USB devices.

## cyusb_programmer

This crate includes a binary for programming a Cypress device. To install:

    cargo install cyusb
    
To run the tool:

    cyusb_programmer [--index <index>] [--target <target>] <image> 
    
Where `<image>` is the path to the firmware file. Currently only the `RAM` target is supported. The `--index` argument can be used to select which device to program if more than one is detected.
