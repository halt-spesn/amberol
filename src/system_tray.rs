// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "windows")]
pub mod windows_tray {
    use gtk::{glib, prelude::*};
    use log::{info, warn, error};
    use std::cell::RefCell;
    use std::rc::Rc;
    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{HICON, LoadIconW},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, 
                NIM_MODIFY, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, 
                LoadCursorW, PostQuitMessage, RegisterClassExW, TranslateMessage, 
                CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG, WM_APP, WM_DESTROY, 
                WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
            },
        },
    };

    const WM_TRAYICON: u32 = WM_APP + 1;

    pub struct SystemTray {
        hwnd: HWND,
        icon_id: u32,
        on_activate: Option<Box<dyn Fn()>>,
    }

    impl SystemTray {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            info!("🔧 Creating Windows system tray");
            
            // Create a hidden window to receive tray messages
            let hwnd = unsafe { Self::create_hidden_window()? };
            
            let mut tray = SystemTray {
                hwnd,
                icon_id: 1,
                on_activate: None,
            };
            
            tray.add_to_tray()?;
            info!("✅ System tray created successfully");
            
            Ok(tray)
        }
        
        pub fn set_on_activate<F>(&mut self, callback: F) 
        where 
            F: Fn() + 'static 
        {
            self.on_activate = Some(Box::new(callback));
        }
        
        unsafe fn create_hidden_window() -> Result<HWND, Box<dyn std::error::Error>> {
            let class_name = windows::core::w!("AmberolTrayClass");
            let window_name = windows::core::w!("AmberolTrayWindow");
            
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?,
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
                lpszMenuName: windows::core::PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: HICON::default(),
            };
            
            RegisterClassExW(&wc);
            
            let hwnd = CreateWindowExW(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
                class_name,
                window_name,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND::default(),
                windows::Win32::UI::WindowsAndMessaging::HMENU::default(),
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?,
                None,
            )?;
            
            Ok(hwnd)
        }
        
        fn add_to_tray(&self) -> Result<(), Box<dyn std::error::Error>> {
            unsafe {
                let mut nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: self.icon_id,
                    uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                    uCallbackMessage: WM_TRAYICON,
                    hIcon: LoadIconW(None, windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION)?,
                    ..Default::default()
                };
                
                // Set tooltip text
                let tooltip = "Amberol - Click to restore";
                let tooltip_wide: Vec<u16> = tooltip.encode_utf16().collect();
                let len = std::cmp::min(tooltip_wide.len(), nid.szTip.len() - 1);
                nid.szTip[..len].copy_from_slice(&tooltip_wide[..len]);
                nid.szTip[len] = 0; // Null terminate
                
                Shell_NotifyIconW(NIM_ADD, &nid)?;
            }
            
            Ok(())
        }
        
        unsafe extern "system" fn window_proc(
            hwnd: HWND, 
            msg: u32, 
            wparam: WPARAM, 
            lparam: LPARAM
        ) -> LRESULT {
            match msg {
                WM_TRAYICON => {
                    match lparam.0 as u32 {
                        WM_LBUTTONUP | WM_RBUTTONUP => {
                            info!("🖱️ Tray icon clicked - attempting to restore window");
                            
                            // Try to restore the main window through GTK
                            glib::MainContext::default().spawn_local(async {
                                if let Some(app) = adw::Application::default() {
                                    if let Some(window) = app.active_window() {
                                        info!("📱 Found active window, presenting...");
                                        window.present();
                                    } else {
                                        warn!("⚠️ No active window found");
                                    }
                                } else {
                                    warn!("⚠️ No default application found");
                                }
                            });
                        }
                        _ => {}
                    }
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                }
                _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
            }
            
            LRESULT(0)
        }
    }
    
    impl Drop for SystemTray {
        fn drop(&mut self) {
            info!("🗑️ Removing system tray icon");
            unsafe {
                let nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: self.icon_id,
                    ..Default::default()
                };
                
                let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod windows_tray {
    pub struct SystemTray;
    
    impl SystemTray {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            Ok(SystemTray)
        }
        
        pub fn set_on_activate<F>(&mut self, _callback: F) 
        where 
            F: Fn() + 'static 
        {
            // No-op on non-Windows platforms
        }
    }
}

pub use windows_tray::SystemTray;