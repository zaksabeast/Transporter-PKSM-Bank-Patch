const fs = require('fs').promises;
const path = require('path');
const { promisify } = require('util');

const exec = promisify(require('child_process').exec);

// Anytime code needs to jump to the patch, it should jump to this offset
const PATCH_ENTRYPOINT_OFFSET = 0x18d5f0;
const READELF = path.join(process.env.DEVKITARM, 'bin', 'arm-none-eabi-readelf');
const CODE_IPS_PATH = 'code.ips';
const RUST_BINARY_PATH = path.join(
  'target',
  'arm-unknown-linux-gnueabi',
  'release',
  'transporter_pksm_patch',
);
const RUST_TEXT_PATH = `${RUST_BINARY_PATH}.text`;
const RUST_RODATA_PATH = `${RUST_BINARY_PATH}.rodata`;

const splitU32 = (num) => {
  return Buffer.from([(num >> 24) & 0xff, (num >> 16) & 0xff, (num >> 8) & 0xff, num & 0xff]);
};

const makeIPSRecord = ({ offset, buf }) => {
  const patchOffset = splitU32(offset).slice(1);
  const size = splitU32(buf.length).slice(2);

  return Buffer.concat([patchOffset, size, buf]);
};

const makeIPS = (records) => {
  const patchStr = Buffer.from('PATCH');
  const eofStr = Buffer.from('EOF');

  return Buffer.concat([patchStr, ...records.map(makeIPSRecord), eofStr]);
};

const getSectionOffset = (readelfOutput, sectionName) => {
  const sectionLine = readelfOutput.split('\n').find((line) => line.includes(sectionName));
  const offsetStr = sectionLine
    .split(' ')
    .filter((str) => str.length > 0)
    .slice(-7)
    .shift();

  return parseInt(offsetStr, 16);
};

const makePatch = async () => {
  const { stdout } = await exec(`${READELF} -S ${RUST_BINARY_PATH}`);

  const textOffset = getSectionOffset(stdout, '.text');
  const rodataOffset = getSectionOffset(stdout, '.rodata');

  const elfBinary = await fs.readFile(RUST_BINARY_PATH, { encoding: null });
  const elfView = new DataView(Uint8Array.from(elfBinary).buffer);
  const entrypointOffset = elfView.getUint32(0x18, true);

  const codePatchOffset = PATCH_ENTRYPOINT_OFFSET - entrypointOffset + textOffset;
  const rodataPatchOffset = PATCH_ENTRYPOINT_OFFSET - entrypointOffset + rodataOffset;

  const textSection = await fs.readFile(RUST_TEXT_PATH, { encoding: null });
  const roPatch = await fs.readFile(RUST_RODATA_PATH, { encoding: null });

  const records = [
    // Check if PKSM Bank exists and is valid
    { offset: 0x148d3c, buf: Buffer.from([0x2b, 0x12, 0x01, 0xeb]) }, // bl #0x448b4    Jump to code patch
    { offset: 0x148d40, buf: Buffer.from([0x60, 0x00, 0x00, 0xea]) }, // b #0x18c       Jump to set next state (returned from code patch)
    // Remove from game save and save to PKSM
    { offset: 0x14a150, buf: Buffer.from([0x47, 0x00, 0x00, 0xea]) }, // b #0x12c       Skip various states
    { offset: 0x14a3d8, buf: Buffer.from([0x84, 0x0c, 0x01, 0xeb]) }, // bl #0x43224    Jump to code patch
    { offset: 0x14a3dc, buf: Buffer.from([0x37, 0x00, 0x00, 0xea]) }, // b #0xf4        Jump to set next state (returned from code patch)
    { offset: 0x142cdc, buf: Buffer.from([0x0f, 0x00, 0xa0, 0xe3]) }, // mov r0, #0xf   Set next state to NO_TRANSFER so bank thinks nothing happened
    { offset: 0x142ce0, buf: Buffer.from([0x10, 0x80, 0xbd, 0xe8]) }, // pop {r4, pc}   Set next state to NO_TRANSFER so bank thinks nothing happened
    // Rust patch
    { offset: codePatchOffset, buf: textSection },
    { offset: rodataPatchOffset, buf: roPatch },
  ];

  const ips = makeIPS(records);

  await fs.writeFile(CODE_IPS_PATH, ips, { encoding: null });

  console.log('IPS file was created!');
};

makePatch();
