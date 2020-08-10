// Can't use == in no_std + linux environment
pub unsafe fn slice_compare(slice1: &[u8], slice2: &[u8]) -> bool {
  let mut i = 0;
  let length = slice1.len();

  while i < length {
    if slice1[i] != slice2[i] {
      return false;
    }

    i += 1;
  }

  return true;
}
