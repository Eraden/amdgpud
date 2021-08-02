use std::io::Error as IoErr;
use std::io::ErrorKind;

#[inline(always)]
pub fn not_found() -> IoErr {
    IoErr::from(ErrorKind::NotFound)
}

#[inline(always)]
pub fn invalid_input() -> IoErr {
    IoErr::from(ErrorKind::NotFound)
}
