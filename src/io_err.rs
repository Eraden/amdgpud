use std::io::Error as IoErr;
use std::io::ErrorKind;

pub fn invalid_data() -> IoErr {
    IoErr::from(ErrorKind::InvalidData)
}

pub fn not_found() -> IoErr {
    IoErr::from(ErrorKind::NotFound)
}

pub fn invalid_input() -> IoErr {
    IoErr::from(ErrorKind::NotFound)
}
