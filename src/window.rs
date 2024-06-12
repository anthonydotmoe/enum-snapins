use log::{debug, trace};
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, IDI_APPLICATION};
use winsafe::{co::{ILC, LVS, LVSIL, SM}, gui, prelude::*, GetSystemMetricsForDpi, HIMAGELIST};
use winsafe::gui::{Horz, Vert};

use crate::snapin::MMCSnapIn;

#[derive(Clone)]
pub struct MyWindow {
    pub wnd: gui::WindowMain,
    pub lv: gui::ListView<()>,
    snapins: Vec<MMCSnapIn>,
}

impl MyWindow {
    pub fn new(snapins: Vec<MMCSnapIn>) -> Self {
        let wnd = gui::WindowMain::new(
            gui::WindowMainOpts {
                title: "EnumSnapins".to_owned(),
                size: (640, 480),
                style: winsafe::co::WS::OVERLAPPEDWINDOW,
                ..Default::default()
            },
        );

        let columns: Vec<(String, u32)> = vec![
            ("Name".to_string(), 300),
            ("Description".to_string(), 300),
            ("CLSID".to_string(), 300)
        ];
        let lv = gui::ListView::new(
            &wnd,
            gui::ListViewOpts {
                size: (640, 480),
                columns,
                list_view_style: LVS::SORTASCENDING | LVS::REPORT,
                resize_behavior: (Horz::Resize, Vert::Resize),
                ..Default::default()
            }
        );

        let new_self = Self {wnd, lv, snapins};
        new_self.events();
        new_self
    }

    fn events(&self) {
        let self2 = self.clone();
        self.wnd.on().wm_create(move |_| {

            // Create imagelist
            let dpi = self2.wnd.hwnd().GetDpiForWindow();
            let icon_cx = GetSystemMetricsForDpi(SM::CXICON, dpi).unwrap();
            let icon_cy = GetSystemMetricsForDpi(SM::CYICON, dpi).unwrap();
            debug!("DPI: {}, Icon size: {}, {}", dpi, icon_cx, icon_cy);
            
            unsafe {
                let small_il = HIMAGELIST::Create(
                    winsafe::SIZE::new(
                        icon_cx,
                        icon_cy,
                    ),
                    ILC::MASK | ILC::COLOR32,
                    self2.snapins.len() as i32,
                    1
                ).unwrap();

                let mut count = 1;

                // Load the placeholder icon (icon index 0)
                let placeholder = LoadIconW(None, IDI_APPLICATION).unwrap();
                let _ = small_il.AddIcon(&winsafe::HICON::from_ptr(placeholder.0 as *mut _));

                for snapin in &self2.snapins {
                    // Add snapin icon if it exists, else placeholder icon
                    trace!("Adding snapin {}", &snapin.namestring.as_ref().unwrap_or(&"".to_string()));
                    let icon_i: u32;
                    if let Some(about) = &snapin.about {
                        if let Some(image) = &about.image {
                            let _ = small_il.AddMasked(&winsafe::HBITMAP::from_ptr(image.large.0 as *mut _), winsafe::COLORREF::from_raw(image.mask.0));
                            icon_i = count;
                            count += 1;
                            trace!("\tAdded image {},\t{:#08x}", icon_i, (image.mask.0 & 0xFFFFFF));
                        } else if let Some(icon) = about.icon {
                            let _ = small_il.AddIcon(&winsafe::HICON::from_ptr(icon.0 as *mut _));
                            icon_i = count;
                            count += 1;
                            trace!("\tAdded icon {}", icon_i);
                        } else {
                            icon_i = 0;
                            trace!("\tUsed placeholder icon {}", icon_i);
                        }
                    } else {
                        icon_i = 0;
                        trace!("\tUsed placeholder icon {}", icon_i);
                    }


                    let snapin_name = snapin.get_name();
                    let snapin_description = snapin.get_description();

                    // Add the listview item to the view for the snapin.
                    self2.lv.items().add(
                        &[snapin_name, snapin_description, &snapin.clsid],
                        Some(icon_i),
                        ()
                    );
                }

                self2.lv.set_image_list(LVSIL::SMALL, small_il);
            }
            Ok(0)
        })



    }
}