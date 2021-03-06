//! Rust binding for [AutoItX](https://www.autoitscript.com/site/autoit/)
//!
//! **(Work in progress):** If you need any function just open an issue or a PR.
//!
//! # Examples
//! ```
//! use autoit::{init, mouse_move, mouse_get_pos};
//!
//! init();
//!
//! mouse_move(0, 0, Some(0));
//! assert_eq!(mouse_get_pos(), (0, 0));
//!
//! mouse_move(50, 50, Some(0));
//! assert_eq!(mouse_get_pos(), (50, 50));
//! ```

extern crate failure;
extern crate widestring;

use failure::Error;
use widestring::WideCString;

use std::char::{decode_utf16, DecodeUtf16Error};
use std::ptr::null;

mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn init() {
    unsafe {
        bindings::AU3_Init();
    }
}

pub fn error() -> i32 {
    // TODO: figure out what AU3_error() is used for
    unsafe { bindings::AU3_error() }
}

pub fn mouse_move(x: i32, y: i32, speed: Option<i32>) {
    // TODO: check return value
    let _ = unsafe { bindings::AU3_MouseMove(x, y, speed.unwrap_or(10)) };
}

pub fn mouse_get_pos() -> (i32, i32) {
    let mut lp = bindings::tagPOINT { x: 0, y: 0 };
    unsafe { bindings::AU3_MouseGetPos(&mut lp) };
    (lp.x, lp.y)
}

pub fn win_exists(title: &str, text: Option<&str>) -> Result<bool, Error> {
    let title_wide = WideCString::from_str(title)?;

    let r = match text {
        Some(t) => {
            let text_wide = WideCString::from_str(t)?;
            unsafe { bindings::AU3_WinExists(title_wide.as_ptr(), text_wide.as_ptr()) }
        }
        None => unsafe { bindings::AU3_WinExists(title_wide.as_ptr(), null()) },
    };

    Ok(r == 1)
}

pub fn win_get_text(
    title: &str,
    text: Option<&str>,
    buf_len: Option<usize>,
) -> Result<String, Error> {
    let title_wide = WideCString::from_str(title)?;

    let buf_len = buf_len.unwrap_or(1024);

    let mut buf = Vec::with_capacity(buf_len as usize);
    let buf_ptr = buf.as_mut_ptr();

    match text {
        Some(t) => {
            let text_wide = WideCString::from_str(t)?;
            unsafe {
                bindings::AU3_WinGetText(
                    title_wide.as_ptr(),
                    text_wide.as_ptr(),
                    buf_ptr,
                    buf_len as i32,
                );
                buf.set_len(buf_len as usize);
            }
        }
        None => unsafe {
            bindings::AU3_WinGetText(title_wide.as_ptr(), null(), buf_ptr, buf_len as i32);
            buf.set_len(buf_len as usize);
        },
    }

    let r = decode_utf16(buf.iter().cloned().take_while(|x| *x != '\0' as u16))
        .collect::<Result<String, DecodeUtf16Error>>()?;

    Ok(r)
}

pub fn win_wait(title: &str, text: Option<&str>, timeout: Option<i32>) -> Result<(), Error> {
    let title_wide = WideCString::from_str(title)?;
    let timeout = timeout.unwrap_or(0);

    match text {
        Some(t) => {
            let text_wide = WideCString::from_str(t)?;
            unsafe { bindings::AU3_WinWait(title_wide.as_ptr(), text_wide.as_ptr(), timeout) };
        }
        None => {
            unsafe { bindings::AU3_WinWait(title_wide.as_ptr(), null(), timeout) };
        }
    };

    Ok(())
}

pub fn set_option(option: &str, value: i32) -> Result<(), Error> {
    let option_wide = WideCString::from_str(option)?;

    unsafe {
        bindings::AU3_AutoItSetOption(option_wide.as_ptr(), value);
    };

    Ok(())
}

pub fn win_get_handle(title: &str, text: Option<&str>) -> Result<*mut bindings::HWND__, Error> {
    let title_wide = WideCString::from_str(title)?;

    let r = match text {
        Some(t) => {
            let text_wide = WideCString::from_str(t)?;
            unsafe { bindings::AU3_WinGetHandle(title_wide.as_ptr(), text_wide.as_ptr()) }
        }
        None => unsafe { bindings::AU3_WinGetHandle(title_wide.as_ptr(), null()) },
    };

    Ok(r)
}

pub fn win_set_on_top(title: &str, text: Option<&str>, flag: i32) -> Result<(), Error> {
    let title_wide = WideCString::from_str(title)?;

    match text {
        Some(t) => {
            let text_wide = WideCString::from_str(t)?;
            unsafe { bindings::AU3_WinSetOnTop(title_wide.as_ptr(), text_wide.as_ptr(), flag) };
        }
        None => {
            unsafe { bindings::AU3_WinSetOnTop(title_wide.as_ptr(), null(), flag) };
        }
    }

    Ok(())
}

pub fn send(keys: &str, flag: Option<i32>) -> Result<(), Error> {
    let keys_wide = WideCString::from_str(keys)?;
    unsafe { bindings::AU3_Send(keys_wide.as_ptr(), flag.unwrap_or(0)) };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Child, Command};

    fn launch_notepad() -> Child {
        Command::new("notepad.exe")
            .arg("tests\\rs-autoit test1.txt")
            .spawn()
            .unwrap()
    }

    #[test]
    fn test_without_notepad() {
        mouse_move(0, 0, Some(0));
        assert_eq!(mouse_get_pos(), (0, 0));

        mouse_move(50, 50, Some(0));
        assert_eq!(mouse_get_pos(), (50, 50));
    }

    #[test]
    fn test_autoit() {
        assert!(!win_exists("rs-autoit test1", None).unwrap());

        let mut notepad = launch_notepad();

        win_wait("rs-autoit test1", None, Some(10));
        win_wait("rs-autoit test1", Some("aéèê"), Some(10));

        assert!(win_exists("rs-autoit test1", None).unwrap());
        assert!(win_exists("rs-autoit test1", Some("aéèê")).unwrap());
        assert!(!win_exists("rs-autoit test1", Some("aéèêT")).unwrap());
        assert_eq!(
            win_get_text("rs-autoit test1", None, None).unwrap(),
            "aéèê\n"
        );
        assert_eq!(
            win_get_text("rs-autoit test1", Some("aéèê"), None).unwrap(),
            "aéèê\n"
        );
        assert_ne!(
            win_get_text("rs-autoit test1", Some("aéèêT"), None).unwrap(),
            "aéèê\n"
        );

        notepad.kill().unwrap();
    }
}
