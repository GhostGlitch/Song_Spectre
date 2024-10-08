mod props;
mod ghoast;
#[cfg(debug_assertions)]
mod debug;
use props::*;
use ghoast::*;
#[allow(unused_imports)]
use windows::{core::*, Data, 
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};
use core::time;
#[allow(unused_imports)]
use std::{io::{Error, ErrorKind}, result::Result};
use futures::executor::block_on;

use std::{sync::{Arc, Mutex}, thread};

async fn get_tcs_manager() -> Result<TCSManager, Error> {
    let manager: TCSManager = TCSManager::RequestAsync()?.get()?;
    Ok(manager)
}
/// Gets the media properties for the provided `TCS` (Global System Media Transport Controls Session).
/// 
/// This function retrieves the media properties for the given `TCS` session, such as title, artist, album, etc.
/// 
/// # Arguments
/// * `sesh` - The `TCS` session to get the media properties for.
/// 
/// # Returns
/// A `Result` containing the `TCSProperties` for the provided `TCS` session, or an `Error` if the operation fails.
async fn get_tcs_props(sesh: TCS) -> Result<TCSProperties, Error> {
    let props: TCSProperties = sesh.TryGetMediaPropertiesAsync()?.get()?;
    Ok(props)
}

fn toast_thread(title: String, props: SpectreProps) {
    thread::spawn(move || {
        let tit = props.title.clone();
        let mut t = debug::show_ghoast(&title, props);
        while t.mloop() {//t.request_paint();
            if t.is_good {
            t.mloop();

            }

        }
        println!("{:?}", tit);
    });
}


fn main() {
    //debug::cls();
    //let mut t = debug::show_ghoast();
    let manager: TCSManager = block_on(get_tcs_manager()).unwrap();
    let sessions = manager.GetSessions().unwrap();
    println!("\n-----------------start-----------------");

    for sesh in sessions.into_iter() {
        let props = block_on(get_tcs_props(sesh)).unwrap();
        let spec_props = SpectreProps::from_tcsp(props);
        let spec_props_thr = spec_props.clone();
        let title = spec_props.title.clone();
        toast_thread(title, spec_props_thr);
        println!("{}", spec_props);

        //#[cfg(debug_assertions)]
        //let _  = debug::display_spec_props(&spec_props);
    }
    thread::sleep(time::Duration::from_secs(20));
    println!("------------------end------------------");
}