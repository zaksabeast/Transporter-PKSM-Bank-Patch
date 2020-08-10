// Thanks to https://libctru.devkitpro.org/fs_8h.html for the types and function signatures
type Handle = u32;
type Result = u32;

const TRANSPORTER_FS_HANDLE: u32 = 0x311f80;
const FSUSER_OPEN_FILE_DIRECTLY: u32 = 0x1df448;
const FSFILE_READ: u32 = 0x15930c;
const FSFILE_WRITE: u32 = 0x159390;
const FSFILE_GET_SIZE: u32 = 0x1593f0;
const FSFILE_CLOSE: u32 = 0x159364;

enum FSArchiveId {
  ArchiveExtData = 6,
  ArchiveSdmc = 9,
}

enum FSPathType {
  PathEmpty = 1,
  PathBinary = 2,
  PathAscii = 3,
}

type FSOpenFileDirectlyPtr = fn(
  fs_handle: *mut Handle,
  file_handle: *mut Handle,
  zero: u32,
  archive_id: FSArchiveId,
  archive_path_type: FSPathType,
  archive_path_data: *const u8,
  archive_path_size: u32,
  file_path_type: FSPathType,
  file_path_data: *const u8,
  file_path_size: u32,
  flags: u32,
  attributes: u32,
) -> Result;

type FSReadPtr = fn(
  handle: *mut Handle,
  bytes_read: *mut u32,
  offset: u64,
  buffer: *mut u8,
  read_size: u32,
) -> Result;

type FSWritePtr = fn(
  handle: *mut Handle,
  bytes_written: *mut u32,
  offset: u64,
  buffer: *mut u8,
  write_size: u32,
  flags: u32,
) -> Result;

type FSGetSizePtr = fn(handle: *mut Handle, size: *mut u64) -> Result;

type FSClosePtr = fn(file_handle: *mut Handle) -> Result;

pub struct File {
  file_handle: Handle,
  pub open_success: bool,
}

impl File {
  pub fn new_sd_file(file_path: &str) -> Self {
    let mut file_handle: Handle = 0;

    let empty_string = "\0";
    let open_result = File::open(
      &mut file_handle,
      0,
      FSArchiveId::ArchiveSdmc,
      FSPathType::PathEmpty,
      empty_string.as_ptr(),
      empty_string.len() as u32,
      FSPathType::PathAscii,
      file_path.as_ptr(),
      file_path.len() as u32,
      3, // OPEN_READ|OPEN_WRITE
      0,
    );

    return File {
      file_handle,
      open_success: open_result == 0,
    };
  }

  pub fn new_extdata_file(extdata_unique_id: u32, file_path: &str) -> Self {
    let mut file_handle: Handle = 0;
    let binary_path = [1, extdata_unique_id, 0];

    let open_result = File::open(
      &mut file_handle,
      0,
      FSArchiveId::ArchiveExtData,
      FSPathType::PathBinary,
      unsafe { core::mem::transmute::<*const u32, *const u8>(binary_path.as_ptr()) },
      core::mem::size_of_val(&binary_path) as u32,
      FSPathType::PathAscii,
      file_path.as_ptr(),
      file_path.len() as u32,
      3, // OPEN_READ|OPEN_WRITE
      0,
    );

    return File {
      file_handle,
      open_success: open_result == 0,
    };
  }

  fn open(
    file_handle: *mut Handle,
    zero: u32,
    archive_id: FSArchiveId,
    archive_path_type: FSPathType,
    archive_path_data: *const u8,
    archive_path_size: u32,
    file_path_type: FSPathType,
    file_path_data: *const u8,
    file_path_size: u32,
    flags: u32,
    attributes: u32,
  ) -> Result {
    unsafe {
      let fs_handle = core::mem::transmute::<u32, *mut u32>(TRANSPORTER_FS_HANDLE);
      let fn_ptr = core::mem::transmute::<u32, FSOpenFileDirectlyPtr>(FSUSER_OPEN_FILE_DIRECTLY);

      return fn_ptr(
        fs_handle,
        file_handle,
        zero,
        archive_id,
        archive_path_type,
        archive_path_data,
        archive_path_size,
        file_path_type,
        file_path_data,
        file_path_size,
        flags,
        attributes,
      );
    }
  }

  pub fn read(&mut self, offset: u64, buffer: &mut [u8]) -> bool {
    let result;
    // Throw away for now
    let mut bytes_read = 0;

    unsafe {
      let fn_ptr = core::mem::transmute::<u32, FSReadPtr>(FSFILE_READ);
      result = fn_ptr(
        &mut self.file_handle,
        &mut bytes_read,
        offset,
        buffer.as_mut_ptr(),
        buffer.len() as u32,
      );
    }

    return result == 0;
  }

  pub fn write(&mut self, offset: u64, buffer: &mut [u8]) -> bool {
    let result;
    // Throw away for now
    let mut bytes_written = 0;

    unsafe {
      let fn_ptr = core::mem::transmute::<u32, FSWritePtr>(FSFILE_WRITE);
      result = fn_ptr(
        &mut self.file_handle,
        &mut bytes_written,
        offset,
        buffer.as_mut_ptr(),
        buffer.len() as u32,
        0,
      );
    }

    return result == 0;
  }

  pub fn get_size(&mut self) -> u64 {
    let mut size = 0;

    unsafe {
      let fn_ptr = core::mem::transmute::<u32, FSGetSizePtr>(FSFILE_GET_SIZE);
      fn_ptr(&mut self.file_handle, &mut size);
    }

    return size;
  }

  pub fn close(&mut self) -> bool {
    let result;

    unsafe {
      let fn_ptr = core::mem::transmute::<u32, FSClosePtr>(FSFILE_CLOSE);
      result = fn_ptr(&mut self.file_handle);
    }

    return result == 0;
  }
}
