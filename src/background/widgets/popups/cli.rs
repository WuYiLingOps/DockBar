pub use dockbar_ipc::commands::PopupsCli;

use crate::error::Result;

pub fn process(cmd: PopupsCli) -> Result<()> {
    let _ = cmd;
    Ok(())
}
