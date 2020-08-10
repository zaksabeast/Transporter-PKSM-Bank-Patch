# Test Steps

This document describes what needs to be tested if changes are made to the patch.

## Transferring Pokemon

Pokemon should correctly transfer under these conditions:

- There are Pokemon in Bank's official Transport box
- There are not Pokemon in Bank's official Transport box
- The origin game is a Gen 1 VC game
- The origin game is a Gen 2 VC game
- The origin game is a Gen 5 cartridge game
- The origin game is a Gen 5 game with the [Transporter save redirect patch](https://github.com/zaksabeast/DreamRadarCartRedirect/releases/tag/v2.0.0) being used
- PKSM is currently using bank files from PKSM's extdata
- PKSM is currently using bank files from the sd card

Verify these happen after transferring a Pokemon:

- Gen 5 Pokemon are transferred as Gen 6 Pokemon
- Gen 1 and 2 Pokemon are transferred as Gen 7 Pokemon

## Legality checks

Yes, I know Transporter legality checks are practically non-existant. The point is that the legality checks still take place and act like they normally would.

- A Pokemon is considered legal by Transporter's legality checks should transfer
- A Pokemon is considered illegal by Transporter's legality checks should not transfer
- A Pokemon in the first slot that is deemed illegal should not affect how other Pokemon are transferred (e.g. they are transferred to an incorrect slot)
- A Pokemon after the first slot and before the last slot is deemed illegal should not affect how other Pokemon are transferred (e.g. they are transferred to an incorrect slot)

## Communication with the Bank server

After transferring with this patch, viewing the Transport box in Pokemon Bank should not show one of these errors:

- "Bank server is locked"
- "Transporter cleanup"

## Error handling

A "Bank has Pokemon in Transport Box" error should occur if:

- The PKSM bank file doesn't exist in extdata or on the SD card
- The PKSM bank file is the wrong version
- The PKSM bank file has the wrong header magic
- The PKSM bank file has too small of a file size
- The PKSM bank file has Pokemon in Box 1
