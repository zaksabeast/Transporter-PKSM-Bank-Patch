// ---------- if PKSM bank updates, these will most likely need to change -----------------------
static BANK_HEADER_MAGIC: [u8; 8] = [0x50, 0x4B, 0x53, 0x4D, 0x42, 0x41, 0x4E, 0x4B];
static SUPPORTED_BANK_VERSION: [u8; 4] = [3, 0, 0, 0];

const PKSM_UNIQUE_ID: u32 = 0xEC100;
const BANK_ENTRY_SIZE: u64 = 0x150;
const BANK_HEADER_SIZE: u64 = 0x10;
const MINIMUM_BANK_SIZE: u64 = BANK_HEADER_SIZE + (BANK_ENTRY_SIZE * EXK_PER_BOX as u64);
const PKSM_GEN_6: [u8; 4] = [2, 0, 0, 0];
const PKSM_GEN_7: [u8; 4] = [3, 0, 0, 0];
// ----------------------------------------------------------------------------------------------

// Also EK6 size
const EK7_SIZE: u32 = 232;
const EXK_PER_BOX: u8 = 30;
const TRANSPORTER_EXK_OFFSET: u32 = 0x8BC6524;
const TRANSPORTER_TRANSFER_SLOT_LIST_OFFSET: u32 = 0x8AFD380;
const TRANSPORTER_GAME_CHECK_OFFSET: u32 = 0x8B0273C;
const GEN_1_2_GAME_CODE: u32 = 2;
// const GEN_5_GAME_CODE: u32 = 0; // Added for documentation, but commented out to make the linter happy

pub struct Bank {
  file: super::game_fs::File,
}

impl Bank {
  pub fn new(path: &str, is_ext_data: bool) -> Self {
    let file = if is_ext_data {
      super::game_fs::File::new_extdata_file(PKSM_UNIQUE_ID, path)
    } else {
      super::game_fs::File::new_sd_file(path)
    };

    return Bank { file };
  }

  pub fn is_valid(&mut self) -> bool {
    // We probably don't need to check if the PKSM box count is greater than 0

    if !self.file.open_success || self.file.get_size() < MINIMUM_BANK_SIZE {
      return false;
    }

    let mut buffer: [u8; 10] = [0; 10];

    self.file.read(0, &mut buffer);

    unsafe {
      // Not pretty, but doesn't need another crate, try_into, or multiple file reads
      // Although I'd love another solution that still fits those requirements
      return super::utils::slice_compare(&buffer[..8], &BANK_HEADER_MAGIC)
        && super::utils::slice_compare(&buffer[8..], &SUPPORTED_BANK_VERSION);
    }
  }

  fn get_slot_offset(&self, slot: u8) -> u64 {
    return (BANK_HEADER_SIZE) + ((slot as u64) * (BANK_ENTRY_SIZE));
  }

  fn is_box_slot_empty(&mut self, slot: u8) -> bool {
    let start_offset = self.get_slot_offset(slot);
    let mut buffer: [u8; 10] = [0; 10];

    // We probably just need to check the sanity bytes
    // Checking the encryption constant, checksum, and species is being extra safe
    self.file.read(start_offset, &mut buffer);

    for byte in buffer.iter() {
      if *byte != 0xff {
        return false;
      }
    }

    return true;
  }

  pub fn is_first_box_empty(&mut self) -> bool {
    let mut box_slot = 0;

    while box_slot < EXK_PER_BOX {
      if !self.is_box_slot_empty(box_slot) {
        return false;
      }

      box_slot += 1;
    }

    return true;
  }

  unsafe fn transfer_slot(&mut self, offset: u32, pksm_box_slot: u8, is_gen_1_or_2: bool) {
    let pksm_offset = self.get_slot_offset(pksm_box_slot);
    let mut generation = if is_gen_1_or_2 {
      PKSM_GEN_7
    } else {
      PKSM_GEN_6
    };

    // Generation data
    self.file.write(pksm_offset, &mut generation);

    let read_ptr = core::mem::transmute::<u32, *mut u8>(offset);
    let mut buffer = core::slice::from_raw_parts_mut(read_ptr, EK7_SIZE as usize);
    self.file.write(pksm_offset + 4, &mut buffer);
  }

  // Transporter has three key lists:
  // - The original potential Pokemon to be transported
  // - Indexes of legal Pokemon in the first list
  // - The EXKs in the order of the second list
  //
  // For example, if index 0 in the first list is illegal,
  // the second list would start with '1'
  // and the third list would start with the EXK of that Pokemon
  //
  // This function checks for non-empty slots in the second list to get offsets of the third list
  unsafe fn get_ekx_offset(&mut self, box_slot: u8) -> u32 {
    let transfer_slot_offset = TRANSPORTER_TRANSFER_SLOT_LIST_OFFSET + (box_slot * 4) as u32;
    let transfer_slot = *core::mem::transmute::<u32, *const u32>(transfer_slot_offset);

    // Check for empty slot
    if transfer_slot == 0xffffffff {
      return 0;
    }

    return TRANSPORTER_EXK_OFFSET + (box_slot as u32 * EK7_SIZE);
  }

  pub fn transfer_box(&mut self) {
    let mut box_slot = 0;

    while box_slot < EXK_PER_BOX {
      unsafe {
        let game_code = *core::mem::transmute::<u32, *const u32>(TRANSPORTER_GAME_CHECK_OFFSET);
        let is_gen_1_or_2 = game_code == GEN_1_2_GAME_CODE;
        let ekx_offset = self.get_ekx_offset(box_slot);

        if ekx_offset != 0 {
          self.transfer_slot(ekx_offset, box_slot, is_gen_1_or_2);
        }
      }

      box_slot += 1;
    }
  }

  pub fn close(&mut self) {
    self.file.close();
  }
}
