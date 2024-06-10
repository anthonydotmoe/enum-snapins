use std::error::Error;
use registry::{Hive, Security};
use window::MyWindow;
use windows::Win32::System::Com::CoInitialize;

mod nsi;
mod snapin;
mod window;

use snapin::MMCSnapIn;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = unsafe { CoInitialize(None) };

    let snapins = get_snapins()?;

    for snapin in snapins.iter() {
        //println!("{:#?}\n", snapin);
    }

    let standalone_snapins: Vec<MMCSnapIn> = snapins.into_iter().filter(|s| s.standalone).collect();

    let my = MyWindow::new(standalone_snapins);

    /*
    my.lv.items().add(
        &[
            "Hello"
        ],
        None,
        (),
    );
    */


    if let Err(e) = my.wnd.run_main(None) {
        eprintln!("{}", e);
    } else {
        return Ok(())
    }

    Ok(())
}

fn get_snapins() -> Result<Vec<MMCSnapIn>, Box<dyn Error>> {
    let mut found_snapins: Vec<MMCSnapIn> = Vec::new();

    // Open HKLM\SOFTWARE\Microsoft\MMC\SnapIns
    let snapins = Hive::LocalMachine.open(
        r"SOFTWARE\Microsoft\MMC\SnapIns",
        Security::Read
    )?;

    // Iterate over each subkey in the SnapIns registry key
    for snapin_key in snapins.keys() {
        let snapin_key = snapin_key?;
        let snapin_clsid = snapin_key.to_string();

        match MMCSnapIn::try_from(snapin_clsid) {
            Ok(snapin) => {
                found_snapins.push(snapin);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    Ok(found_snapins)
}
