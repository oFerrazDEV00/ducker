/// Caixa de diálogo nativa do sistema operacional para confirmação de recebimento
pub fn prompt_user_acceptance(title: &str, message: &str) -> bool {
    if std::env::var("DUCKER_NO_PROMPT").is_ok() || cfg!(test) {
        return true;
    }
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy().to_lowercase();
        if exe_str.contains("deps\\") || exe_str.contains("deps/") || exe_str.contains("test") {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(
                hwnd: *mut std::ffi::c_void,
                lp_text: *const u16,
                lp_caption: *const u16,
                u_type: u32,
            ) -> i32;
        }

        const MB_YESNO: u32 = 0x00000004;
        const MB_ICONQUESTION: u32 = 0x00000020;
        const MB_TOPMOST: u32 = 0x00040000;
        const MB_SETFOREGROUND: u32 = 0x00010000;
        const IDYES: i32 = 6;

        let text_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();
        let caption_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();

        let result = unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                text_wide.as_ptr(),
                caption_wide.as_ptr(),
                MB_YESNO | MB_ICONQUESTION | MB_TOPMOST | MB_SETFOREGROUND,
            )
        };

        result == IDYES
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (title, message);
        true
    }
}
