use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, LoadIconW, IDI_APPLICATION, SM_CXICON, SM_CYICON};
use winsafe::{co::{ILC, LVS, LVSIL}, gui, prelude::*, HIMAGELIST};

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
                title: "My window title".to_owned(),
                size: (640, 480),
                ..Default::default()
            },
        );

        let mut columns: Vec<(String, u32)> = Vec::new();
        columns.push(("Name".to_string(), 300));
        println!("Columns\n{:?}", columns);
        let lv = gui::ListView::new(
            &wnd,
            gui::ListViewOpts {
                size: (640, 480),
                columns,
                list_view_style: LVS::SORTASCENDING | LVS::REPORT,
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
            unsafe {
                let small_il = HIMAGELIST::Create(
                    winsafe::SIZE::new(
                        GetSystemMetrics(SM_CXICON),
                        GetSystemMetrics(SM_CYICON),
                    ),
                    ILC::MASK | ILC::COLOR32,
                    self2.snapins.len() as i32,
                    1
                ).unwrap();

                //println!("GetSystemMetrics: {}", GetSystemMetrics(SM_CXICON));

                let mut count = 1;

                // Load the placeholder icon (icon index 0)
                let placeholder = LoadIconW(None, IDI_APPLICATION).unwrap();
                let _ = small_il.AddIcon(&winsafe::HICON::from_ptr(placeholder.0 as *mut _));

                for snapin in &self2.snapins {
                    // Add snapin icon if it exists, else placeholder icon
                    let icon_i: u32;
                    if let Some(about) = &snapin.about {
                        if let Some(icon) = about.icon {
                            let _ = small_il.AddIcon(&winsafe::HICON::from_ptr(icon.0 as *mut _));
                            icon_i = count;
                            count += 1;
                            println!("Added icon {}", icon_i);
                        } else if let Some(image) = &about.image {
                            let _ = small_il.Add(&winsafe::HBITMAP::from_ptr(image.small.0 as *mut _), None);
                            icon_i = count;
                            count += 1;
                            println!("Added image {}", icon_i);
                        } else {
                            icon_i = 0;
                            println!("Used placeholder icon {}", icon_i);
                        }
                    } else {
                        icon_i = 0;
                        println!("Used placeholder icon {}", icon_i);
                    }

                    // Get snapin name if it exists, else blank
                    let snapin_name: String;
                    if let Some(name) = &snapin.namestring {
                        snapin_name = name.clone();
                    }
                    else if let Some(name) = &snapin.namestringindirect {
                        snapin_name = name.clone();
                    }
                    else {
                        snapin_name = "".to_string();
                    }

                    // Add the listview item to the view for the snapin.
                    self2.lv.items().add(
                        &[&snapin_name],
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