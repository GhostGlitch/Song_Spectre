mod props;
#[cfg(debug_assertions)]
mod debug;
use props::*;
#[allow(unused_imports)]
use windows::{core::*, Data, 
    Media::{ MediaPlaybackType as MPT, 
        Control::{
            GlobalSystemMediaTransportControlsSession as TCS, GlobalSystemMediaTransportControlsSessionManager as TCSManager, GlobalSystemMediaTransportControlsSessionMediaProperties as TCSProperties, *}}};
#[allow(unused_imports)]
use std::{io::{Error, ErrorKind}, result::Result};
use futures::executor::block_on;

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
fn main() {
    let manager: TCSManager = block_on(get_tcs_manager()).unwrap();
    let sessions = manager.GetSessions().unwrap();
    println!("\n-----------------start-----------------");
    for sesh in sessions.into_iter() {
        let props = block_on(get_tcs_props(sesh)).unwrap();
        let spec_props = SpectreProps::from_tcsp(props);
        #[cfg(debug_assertions)]
        let _  = debug::display_spec_props(&spec_props);
    }
    println!("------------------end------------------")
}