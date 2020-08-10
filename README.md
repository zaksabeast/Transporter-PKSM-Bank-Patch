# Transporter PKSM bank patch

This is a 3DS patch (mostly) written in Rust that patches Pokemon Transporter to transfer Pokemon to PKSM's bank.

Use this at your own risk. I'm not responsible if anything bad happens.

## Why did I make this

For the most part I wanted to learn more about rust and write a console patch in rust.

Aside from that, having Transporter dump Gen 1 and 2 Pokemon to PKSM is kind of cool.

## Features

- Gen 1, 2, and 5 Pokemon can be sent to PKSM's Bank as Gen 7 Pokemon
- The patch can use a PKSM bank in PKSM's extdata or on the SD card
- The patch works with the [Transporter save redirect patch](https://github.com/zaksabeast/DreamRadarCartRedirect/releases/tag/v2.0.0)

## Caveats

Normally Transporter sends Pokemon to bank, and bank sends Pokemon to a game. This means Transporter doesn't have the ability to set certain properties, such as the Pokemon's handling trainer, because it doesn't know what game the Pokemon will be sent to.

This is a list of properties Transporter can't handle or handles incorrectly, and Bank usually fixes:

- Handling trainer information is not set
- Geolocation information is not set
- The PP of a some moves appears to be set incorrectly

Since we're using PKSM instead of Bank, the sections below will address how to fix these.

## Setting up

1. Download the `code.ips` file
1. Put the `code.ips` file on your 3DS sd card at `/luma/titles/00040000000C9C00/code.ips`
1. Open PKSM and enable the "Edit during transfers" setting
   - This will set the correct handling trainer information when moving to a Gen 7 game
1. Create a new PKSM bank called "transporter"

## Usage

1. Open Transporter and transport Pokemon like normal
1. Open PKSM and move the Pokemon to a Gen 7 game
1. Open the Gen 7 game and move the Pokemon into your party
   - Moving the Pokemon into your party sets the PP correctly
1. Use PKSM to set the geolocation info, such as region. [Guidance is provided here](https://github.com/FlagBrew/PKSM/issues/1195#issuecomment-647627193)
   - Transporter doesn't set Geolocation info

## Contributing

Contributions are 100% welcome. If you decide to contribute:

- Please open an issue first to discuss potential changes
- Before making a pull request, please test as much as you can in the `TEST_STEPS.md` file
  - For example, if you don't have a Gen 5 game to test with, someone else can test that during the pull request review process
- If everything looks good, a pull request is welcome

## Building

You'll need the following to build:

- devkitarm
- rust toolchain
- node.js

After installing those, run `make`.

## FAQ

**Q: Why does Transporter say, "At least one Pokemon remains in the Transport Box from your previous session"?**

A: You have one of these issues:

- You don't have a PKSM bank named "transporter"
- You have Pokemon in Box 1 of your PKSM "transporter" bank
- Your PKSM "transporter" bank is invalid

**Q: My transferred Pokemon is illegal, can you help?**

A: Please open an issue on this repository, and I can take a look at potential issues

**Q: Are you going to release a patch to make Transporter work offline?**

A: While this patch does exist in private, it's not something I intend to release in the near future.

**Q: Why did you do <X> and how does <Y> work in Transporter?**

A: I'd be happy to answer these types of questions on Discord. Please reach out to zaksabeast#7423 on the PokemonRNG Discord (or any other Discord place you see me in).
