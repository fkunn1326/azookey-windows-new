#[macro_export]
macro_rules! check_win32_err {
    // macro to convert error from HRESULT to windows::core::Error
    ($result:ident) => {
        if $result.is_ok() {
            return Ok(());
        } else {
            return Err(windows::core::Error::from($result.to_hresult()));
        }
    };
    ($result:ident, $value:ident) => {
        if $result.is_ok() {
            Ok($value)
        } else {
            Err(windows::core::Error::from($result.to_hresult()))
        }
    };
}

#[macro_export]
macro_rules! check_err {
    // macros to convert error from anyhow to HRESULT
    ($result:ident) => {
        if $result.is_ok() {
            windows::Win32::Foundation::S_OK
        } else {
            log::error!("{:?}", $result.err());
            windows::Win32::Foundation::S_FALSE
        }
    };

    ($result:ident, $error:ident) => {
        if $result.is_ok() {
            windows::Win32::Foundation::S_OK
        } else {
            log::error!("{:?}", $result.err());
            $error
        }
    };

    ($result:ident, $ok:ident, $error:ident) => {
        if $result.is_ok() {
            $ok
        } else {
            log::error!("{:?}", $result.err());
            $error
        }
    };
}
