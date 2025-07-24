// SPDX-FileCopyrightText: 2024  Emmanuele Bassi
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(target_os = "windows")]
pub mod windows_tray {
    use gtk::{glib, prelude::*};
    use log::{info, warn, error};
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::icon_renderer::IconRenderer;
    use windows::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM, HINSTANCE, POINT},
        Graphics::Gdi::HBRUSH,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, 
                NIM_MODIFY, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow, LoadCursorW, PostQuitMessage, 
                RegisterClassExW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, 
                WM_APP, WM_DESTROY, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW, 
                WS_OVERLAPPEDWINDOW, HICON, LoadIconW, IDI_APPLICATION, WINDOW_EX_STYLE,
                HMENU, LoadImageW, IMAGE_ICON, LR_LOADFROMFILE,
                CreatePopupMenu, AppendMenuW, TrackPopupMenu, DestroyMenu, SetForegroundWindow,
                MF_STRING, TPM_RIGHTBUTTON, TPM_RETURNCMD, WM_COMMAND, GetCursorPos,
            },
        },
    };

    const WM_TRAYICON: u32 = WM_APP + 1;

    pub struct SystemTray {
        hwnd: HWND,
        icon_id: u32,
        custom_icon: Option<HICON>,
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
            
            let mut tray = SystemTray {
                hwnd,
                icon_id: 1,
                custom_icon: None,
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
            
            let hinstance = GetModuleHandleW(None)?;
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance.into(),
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
                hinstance,
                None,
            )?;
            
            Ok(hwnd)
        }
        
        fn add_to_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            unsafe {
                let mut nid = NOTIFYICONDATAW {
                    cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: self.icon_id,
                    uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                    uCallbackMessage: WM_TRAYICON,
                    hIcon: {
                        // Try to use our custom tray icon, fallback to default
                        use crate::icon_renderer::IconRenderer;
                        if let Some(custom_icon) = IconRenderer::create_tray_icon() {
                            info!("🎨 Using custom tray icon");
                            self.custom_icon = Some(custom_icon);
                            custom_icon
                        } else {
                            warn!("⚠️ Failed to create custom tray icon, using default");
                            LoadIconW(None, IDI_APPLICATION)?
                        }
                    },
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
                        WM_LBUTTONUP => {
                            info!("🖱️ Tray icon left-clicked - restoring window");
                            
                            glib::idle_add_once(|| {
                                info!("📱 Tray clicked - sending restore signal");
                                
                                // Try to find any GTK application and trigger an action
                                if let Some(app) = gtk::gio::Application::default() {
                                    app.activate();
                                    info!("📱 Activated application via GApplication");
                                } else {
                                    warn!("⚠️ Could not find GApplication to activate");
                                }
                                
                                glib::ControlFlow::Continue
                            });
                        }
                        WM_RBUTTONUP => {
                            info!("🖱️ Tray icon right-clicked - showing context menu");
                            Self::show_context_menu(hwnd);
                        }
                        _ => {
                            // Handle other tray messages if needed
                        }
                    }
                }
                WM_COMMAND => {
                    let command_id = (wparam.0 & 0xFFFF) as u32;
                    match command_id {
                        1001 => {
                            // Restore/Show window
                            info!("📱 Context menu: Restore selected");
                            glib::idle_add_once(|| {
                                if let Some(app) = gtk::gio::Application::default() {
                                    app.activate();
                                }
                                glib::ControlFlow::Continue
                            });
                        }
                        1002 => {
                            // Quit application
                            info!("🚪 Context menu: Quit selected");
                            glib::idle_add_once(|| {
                                if let Some(app) = gtk::gio::Application::default() {
                                    app.quit();
                                    info!("📱 Application quit requested");
                                }
                                glib::ControlFlow::Continue
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
        
        /// Show context menu for tray icon
        unsafe fn show_context_menu(hwnd: HWND) {
            let hmenu = CreatePopupMenu();
            if hmenu.is_invalid() {
                warn!("Failed to create popup menu");
                return;
            }
            
            // Add menu items
            let restore_text: Vec<u16> = "Show Amberol\0".encode_utf16().collect();
            let quit_text: Vec<u16> = "Quit\0".encode_utf16().collect();
            
            AppendMenuW(hmenu, MF_STRING, 1001, windows::core::PCWSTR(restore_text.as_ptr()));
            AppendMenuW(hmenu, MF_STRING, 1002, windows::core::PCWSTR(quit_text.as_ptr()));
            
            // Get cursor position
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            
            // Required for proper menu behavior
            SetForegroundWindow(hwnd);
            
            // Show menu and get selection
            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RIGHTBUTTON | TPM_RETURNCMD,
                pt.x,
                pt.y,
                0,
                hwnd,
                None,
            );
            
            // Handle menu selection
            if cmd != 0 {
                use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
                SendMessageW(hwnd, WM_COMMAND, WPARAM(cmd as usize), LPARAM(0));
            }
            
            DestroyMenu(hmenu);
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
                
                // Clean up custom icon if we created one
                if let Some(icon) = self.custom_icon {
                    let _ = DestroyIcon(icon);
                    info!("🗑️ Cleaned up custom tray icon");
                }
                
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