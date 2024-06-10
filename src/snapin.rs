use std::{error::Error, str::FromStr};

use registry::{self as reg, Hive, RegKey, Security};
use windows::core::{IUnknown, Interface, GUID, PCWSTR, PWSTR};
use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::WindowsAndMessaging::{CopyIcon, CopyImage, DestroyIcon, IMAGE_BITMAP, IMAGE_FLAGS};
use windows::Win32::{
    System::{
        Com::{
            CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER
        },
        LibraryLoader::LoadLibraryW,
        Mmc::ISnapinAbout
    },
    UI::WindowsAndMessaging::{LoadStringW, HICON},
};
use windows::Win32::Graphics::Gdi::{self, DeleteObject, HBITMAP};


use crate::nsi;

#[derive(Clone, Default)]
pub struct MMCSnapIn {
    pub clsid: String,
    pub about: Option<MMCSnapInAbout>,
    //pub filename: PathBuf,
    pub namestring: Option<String>,
    pub description: Option<String>,
    pub namestringindirect: Option<String>,
    pub standalone: bool,
    pub providerstringindirect: Option<String>,
    pub versionstringindirect: Option<String>,
    pub application_base: Option<String>,
    pub module_name: Option<String>,
}

#[derive(Clone, Default)]
pub struct MMCSnapInAbout {
    pub description: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub icon: Option<HICON>,
    pub image: Option<MMCSnapInImage>,
}

impl Drop for MMCSnapInAbout {
    fn drop(&mut self) {
        if let Some(icon) = self.icon {
            if !icon.is_invalid() {
                let _ = unsafe { DestroyIcon(icon) };
            }
        }
    }
}
#[derive(Clone, Default)]
pub struct MMCSnapInImage {
    pub small: HBITMAP,
    pub small_open: HBITMAP,
    pub large: HBITMAP,
    pub mask: COLORREF,
}

impl Drop for MMCSnapInImage {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.small);
            let _ = DeleteObject(self.small_open);
            let _ = DeleteObject(self.large);
        }
    }
}

trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl ToWide for str {
    fn to_wide(&self) -> Vec<u16> {
        self.encode_utf16().collect()
    }

    fn to_wide_null(&self) -> Vec<u16> {
        self.encode_utf16().chain(Some(0)).collect()
    }
}

impl TryFrom<String> for MMCSnapIn {
    type Error = Box<dyn Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let regpath = format!("SOFTWARE\\Microsoft\\MMC\\SnapIns\\{}", value);
        let regkey: RegKey;
        match Hive::LocalMachine.open(
            &regpath,
            Security::Read
        ) {
            Err(e) => {
                return Err(e.into());
            }
            Ok(key) => {
                regkey = key;
            }
        }

        let mut snapin = MMCSnapIn {
            clsid: value,
            ..Default::default()
        };

        snapin.standalone = regkey
            .keys()
            .any(|k| {
                k.map(|key| key.to_string().eq_ignore_ascii_case("StandAlone"))
                    .unwrap_or(false)
            });

        for value in regkey.values() {
            let value = value?;
            match value.name().to_string_lossy().as_str() {
                "About" => {
                    if let reg::Data::String(data) = value.data() {
                        
                        let id = uuid::Uuid::parse_str(data.to_string_lossy().as_str())?;
                        let clsid = GUID::from_values(
                            id.as_fields().0,
                            id.as_fields().1,
                            id.as_fields().2,
                            id.as_fields().3.clone()
                        );
                        if let Ok(about) = MMCSnapInAbout::try_from(clsid) {
                            snapin.about = Some(about);
                        }
                    }
                },
                "NameString" => {
                    if let reg::Data::String(data) = value.data() {
                        snapin.namestring = Some(data.to_string_lossy());
                    }
                },
                "NameStringIndirect" => {
                    if let reg::Data::String(data) = value.data() {
                        match nsi::IndirectString::from_str(data.to_string_lossy().as_str()) {
                            Err(_) => {},
                            Ok(nsi) => {
                                let dllpath = nsi.dllpath;
                                match load_dll_string(&dllpath, nsi.strid) {
                                    Err(_) => {},
                                    Ok(namestring) => {
                                        snapin.namestringindirect = Some(namestring);
                                    }
                                }
                                
                            }
                        }
                    }
                },
                "ProviderStringIndirect" => {
                    if let reg::Data::String(data) = value.data() {
                        match nsi::IndirectString::from_str(data.to_string_lossy().as_str()) {
                            Err(_) => {},
                            Ok(nsi) => {
                                let dllpath = nsi.dllpath;
                                match load_dll_string(&dllpath, nsi.strid) {
                                    Err(_) => {},
                                    Ok(provider) => {
                                        snapin.providerstringindirect = Some(provider);
                                    }
                                }
                                
                            }
                        }
                    }
                },
                "VersionStringIndirect" => {
                    if let reg::Data::String(data) = value.data() {
                        match nsi::IndirectString::from_str(data.to_string_lossy().as_str()) {
                            Err(_) => {},
                            Ok(nsi) => {
                                let dllpath = nsi.dllpath;
                                match load_dll_string(&dllpath, nsi.strid) {
                                    Err(_) => {},
                                    Ok(version) => {
                                        snapin.versionstringindirect = Some(version);
                                    }
                                }
                                
                            }
                        }
                    }
                },
                "ApplicationBase" => {
                    if let reg::Data::String(data) = value.data() {
                        snapin.application_base = Some(data.to_string_lossy());
                    }
                },
                "ModuleName" => {
                    if let reg::Data::String(data) = value.data() {
                        snapin.module_name = Some(data.to_string_lossy());
                    }
                },
                "Description" => {
                    if let reg::Data::String(data) = value.data() {
                        snapin.description = Some(data.to_string_lossy());
                    }
                },
                _ => {},
            }
        }

        Ok(snapin)
    }
}

impl TryFrom<GUID> for MMCSnapInAbout {
    type Error = Box<dyn Error>;

    fn try_from(value: GUID) -> Result<Self, Self::Error> {
        let mut snapin_about = MMCSnapInAbout::default();
        unsafe {
            match CoCreateInstance::<_, IUnknown>(&value, None, CLSCTX_INPROC_SERVER) {
                Ok(iunk) => {
                    let about = iunk.cast::<ISnapinAbout>();
                    if let Ok(about) = about {
                        //println!("Created instance for {:?}", value);
                        // Get description
                        //println!("\tGetSnapinDescription()");
                        let desc_ptr = about.GetSnapinDescription()?;
                        if !desc_ptr.is_null() {
                            let desc = desc_ptr.to_string()?;
                            //println!("\tGot {} at {:#x}, freeing", desc, desc_ptr.0 as usize);
                            CoTaskMemFree(Some(desc_ptr.0 as *const _));
                            snapin_about.description = Some(desc);
                        }

                        // Get provider
                        //println!("\tGetProvider()");
                        let prov_ptr = about.GetProvider()?;
                        if !prov_ptr.is_null() {
                            let prov = prov_ptr.to_string()?;
                            //println!("\tGot {} at {:#x}, freeing", prov, prov_ptr.0 as usize);
                            CoTaskMemFree(Some(prov_ptr.0 as *const _));
                            snapin_about.provider = Some(prov);
                        }

                        // Get version
                        //println!("\tGetSnapinVersion()");
                        let ver_ptr = about.GetSnapinVersion()?;
                        if !ver_ptr.is_null() {
                            //println!("\tGot {:?} from GetSnapinVersion()", ver_ptr);
                            let ver = ver_ptr.to_string()?;
                            //println!("\tGot {} at {:#x}, freeing", ver, ver_ptr.0 as usize);
                            CoTaskMemFree(Some(ver_ptr.0 as *const _));
                            snapin_about.version = Some(ver);
                        }

                        // Get icon
                        let icon_ptr = about.GetSnapinImage()?;
                        if !icon_ptr.is_invalid() {
                            let my_icon = CopyIcon(icon_ptr)?;
                            snapin_about.icon = Some(my_icon);
                        }

                        // Get images

                        let mut small = Gdi::HBITMAP(0);
                        let mut small_open = Gdi::HBITMAP(0);
                        let mut large = Gdi::HBITMAP(0);
                        let mut cmask = windows::Win32::Foundation::COLORREF(0);
                        match about.GetStaticFolderImage(
                            &mut small, 
                            &mut small_open, 
                            &mut large, 
                            &mut cmask
                        ) {
                            Ok(_) => {
                                let image = MMCSnapInImage {
                                    small: copy_bitmap(small),
                                    small_open: copy_bitmap(small_open),
                                    large: copy_bitmap(large),
                                    mask: cmask.clone(),
                                };

                                snapin_about.image = Some(image);
                            }
                            Err(_) => {

                            }
                        }
                        // Ref counting?
                        //drop(about_ref);

                        // Return filled struct
                        Ok(snapin_about)
                    }
                    else {
                        Err("ISnapInAbout not supported".into())
                    }
                }
                Err(e) => Err(e.into())
            }
        }
        
    }
}

fn load_dll_string(dll_path: &str, str_id: i32) -> Result<String, Box<dyn Error>> {
    unsafe {
        let h_module = LoadLibraryW(PCWSTR(dll_path.to_wide_null().as_ptr()))?;

        let mut buffer: [u16; 260] = [0; 260];
        let length = LoadStringW(h_module, str_id as u32, PWSTR(buffer.as_mut_ptr()), buffer.len() as i32);

        if length == 0 {
            return Err("Failed to load string resource".into());
        }

        let string = String::from_utf16_lossy(&buffer[..length as usize]);
        Ok(string)
    }
}

fn copy_bitmap(src: HBITMAP) -> HBITMAP {
    let src_h = windows::Win32::Foundation::HANDLE(src.0);
    let dst_h = unsafe { CopyImage(src_h, IMAGE_BITMAP, 0, 0, IMAGE_FLAGS(0)).unwrap() };
    HBITMAP(dst_h.0)
}