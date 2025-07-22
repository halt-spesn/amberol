// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "windows")]
pub mod windows_tray {
    use gtk::{glib, prelude::*};
    use log::{info, warn, error};
    use std::cell::RefCell;
    use std::rc::Rc;
    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM, HINSTANCE},
        Graphics::Gdi::HBRUSH,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, 
                NIM_MODIFY, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, LoadCursorW, PostQuitMessage, 
                RegisterClassExW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, 
                WM_APP, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW, 
                WS_OVERLAPPEDWINDOW, HICON, LoadIconW, IDI_APPLICATION, WINDOW_EX_STYLE,
                HMENU,
            },
        },
    };

    const WM_TRAYICON: u32 = WM_APP + 1;

    pub struct SystemTray {
        hwnd: HWND,
        icon_id: u32,
    }

    impl std::fmt::Debug for SystemTray {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SystemTray")
                .field("icon_id", &self.icon_id)
                .finish()
        }
    }

    impl SystemTray {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            info!("🔧 Creating Windows system tray");
            
            // Create a hidden window to receive tray messages
            let hwnd = unsafe { Self::create_hidden_window()? };
            
            let tray = SystemTray {
                hwnd,
                icon_id: 1,
            };
            
            tray.add_to_tray()?;
            info!("✅ System tray created successfully");
            
            Ok(tray)
        }
        
        pub fn set_on_activate<F>(&mut self, _callback: F) 
        where 
            F: Fn() + 'static 
        {
            // Callback is no longer stored, tray activation is handled directly
            // This method is kept for API compatibility
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
                hInstance: GetModuleHandleW(None)?.into(),
                hIcon: HICON::default(),
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: HBRUSH::default(),
                lpszMenuName: windows::core::PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: HICON::default(),
            };
            
            RegisterClassExW(&wc);
            
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                window_name,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND::default(),
                HMENU::default(),
                GetModuleHandleW(None)?.into(),
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
                    hIcon: LoadIconW(None, IDI_APPLICATION)?,
                    ..Default::default()
                };
                
                // Set tooltip text
                let tooltip = "Amberol - Click to restore";
                let tooltip_wide: Vec<u16> = tooltip.encode_utf16().collect();
                let len = std::cmp::min(tooltip_wide.len(), nid.szTip.len() - 1);
                nid.szTip[..len].copy_from_slice(&tooltip_wide[..len]);
                nid.szTip[len] = 0; // Null terminate
                
                let result = Shell_NotifyIconW(NIM_ADD, &nid);
                if result.as_bool() == false {
                    return Err("Failed to add system tray icon".into());
                }
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
                            info!("🖱️ Tray icon clicked - will restore window via GAction");
                            
                            // Use a simpler approach: trigger a GAction that can be handled by the application
                            glib::idle_add_once(|| {
                                // Look for any GTK application and send a restore action
                                if let Some(display) = gtk::gdk::Display::default() {
                                    if let Some(app) = gtk::gio::Application::default() {
                                        if let Some(window) = app.active_window() {
                                            info!("📱 Restoring window from tray");
                                            window.set_visible(true);
                                            window.present();
                                            if let Ok(gtk_window) = window.downcast::<gtk::ApplicationWindow>() {
                                                gtk_window.activate();
                                            }
                                            return;
                                        }
                                    }
                                }
                                warn!("⚠️ Could not find GTK application to restore");
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
    #[derive(Debug)]
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