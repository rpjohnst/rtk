extern crate winapi;
extern crate user32;
extern crate kernel32;
extern crate gdi32;
extern crate comctl32;
extern crate libc;

use std::mem;
use std::ptr;
use std::fmt;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use self::winapi::*;

// This is a hack to force the .rsrc section to stick around even when linked from an (this) rlib.
// It happens to defeat the optimizer for now, but we probably need a more direct solution later.
//
// Win32 executables need to link with a manifest resource (see lib/manifest-sys) to tell Windows
// they support comctl32.dll 6.0 or they'll be stuck with old-style controls, so this links with
// the manifest resource and forces a dependency on a symbol inserted into the object so ld won't
// skip it while linking librtk.rlib with the final executable.

#[repr(C)]
struct Dead;
impl fmt::Debug for Dead {
    fn fmt(&self, _: &mut fmt::Formatter) -> Result<(), fmt::Error> { Ok(()) }
}

#[link(name = "manifest", kind = "static")]
extern {
    static rsrc: Dead;
}

pub struct App {
    _private: (),
}

impl App {
    pub fn new() -> App {
        // this is a hack to keep the .rsrc section around; see above
        println!("{:?}", rsrc);

        let icc = INITCOMMONCONTROLSEX {
            dwSize: mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD,
            dwICC: ICC_STANDARD_CLASSES,
        };
        unsafe { comctl32::InitCommonControlsEx(&icc); }
        App { _private: () }
    }

    pub fn new_window(&self, title: &str, width: i32, height: i32) -> Result<Window, WindowError> {
        unsafe extern "system" fn wnd_proc(
            hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM
            ) -> LRESULT {
            match msg {
                // TODO: handle signals
                //WM_COMMAND => {}

                WM_DESTROY => { user32::PostQuitMessage(0); 0 }
                _ => user32::DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        let class_name: Vec<_> = OsStr::new("RTK\0").encode_wide().collect();

        let class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: unsafe { kernel32::GetModuleHandleW(ptr::null()) },
            hIcon: unsafe { user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION) },
            hCursor: unsafe { user32::LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: ptr::null_mut(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: unsafe { user32::LoadIconW(ptr::null_mut(), IDI_APPLICATION) },
        };

        if unsafe { user32::RegisterClassExW(&class) } == 0 {
            return Err(WindowError::Create);
        }

        let title: Vec<_> = OsStr::new(title).encode_wide().chain(Some(0).into_iter()).collect();

        let hwnd = unsafe { user32::CreateWindowExW(
                WS_EX_CLIENTEDGE,
                class_name.as_ptr(),
                title.as_ptr() as LPCWSTR,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, CW_USEDEFAULT,
                width, height,
                ptr::null_mut(), ptr::null_mut(),
                kernel32::GetModuleHandleW(ptr::null()), ptr::null_mut()
                ) };

        if hwnd == ptr::null_mut() {
            return Err(WindowError::Create);
        }

        Ok(Window { hwnd: hwnd })
    }

    pub fn run(&self) {
        unsafe {
            loop {
                let mut msg = mem::uninitialized();
                if user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) == 0 { break; }

                user32::TranslateMessage(&msg);
                user32::DispatchMessageW(&msg);
            }
        }
    }
}

#[repr(C)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn show(&self) {
        unsafe { user32::ShowWindow(self.hwnd, SW_SHOWDEFAULT); }
    }

    pub fn new_button(&self, x: i32, y: i32, text: &str) -> Button {
        let class: Vec<_> = OsStr::new("BUTTON\0").encode_wide().collect();
        let text: Vec<_> = OsStr::new(text).encode_wide().chain(Some(0).into_iter()).collect();
        let button = unsafe { user32::CreateWindowExW(
            0, class.as_ptr(), text.as_ptr(),
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
            x, y, 100, 24,
            self.hwnd, ptr::null_mut(),
            kernel32::GetModuleHandleW(ptr::null()), ptr::null_mut()
        ) };

        unsafe {
            let mut ncm: NONCLIENTMETRICSW = mem::uninitialized();
            ncm.cbSize = mem::size_of::<NONCLIENTMETRICSW>() as UINT;
            user32::SystemParametersInfoW(
                SPI_GETNONCLIENTMETRICS, ncm.cbSize, mem::transmute(&mut ncm), 0
                );
            let font = gdi32::CreateFontIndirectW(&mut ncm.lfMessageFont);
            user32::SendMessageW(button, WM_SETFONT, font as WPARAM, 0);
        }

        Button { hwnd: button }
    }
}

#[repr(C)]
pub struct Button {
    hwnd: HWND,
}

#[derive(Debug)]
pub enum WindowError {
    Create,
}
