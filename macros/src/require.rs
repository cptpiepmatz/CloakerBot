#[macro_export]
macro_rules! require {
    (Some($unwrap:expr)$( => $ret:expr)?) => {
        match $unwrap {
            Some(inner) => inner,
            None => return $($ret)?,
        }
    };

    (Ok($unwrap:expr)$( => $ret:expr)?) => {
        match $unwrap {
            Ok(inner) => inner,
            Err(_) => return $($ret)?,
        }
    };

    ($cond:expr$( => $ret:expr)?) => {
        if !($cond) {
            return $($ret)?;
        }
    };
}
