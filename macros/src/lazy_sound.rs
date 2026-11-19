#[macro_export]
macro_rules! lazy_sound {
    ($ident:ident) => {
        ::std::sync::LazyLock::new(|| {
            use crate::diagnostics::panic::ExpectReport;
            crate::takedown::sounds::Sound::from_static_wav_bytes(crate::assets::sounds::$ident)
                .expect_report(concat!("could not load ", stringify!($ident), " sound"))
        })
    };

    ($ident:ident+) => {
        ::std::sync::LazyLock::new(|| {
            use crate::diagnostics::panic::ExpectReport;
            crate::assets::sounds::$ident
                .into_iter()
                .enumerate()
                .map(|(i, bytes)| {
                    crate::takedown::sounds::Sound::from_static_wav_bytes(bytes)
                        .expect_report(format!("could not load {}[{i}] sound", stringify!($ident)))
                })
                .collect()
        })
    };
}
