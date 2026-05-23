use super::{Write, io};
pub fn beep() {
    print!("\x07");
    let _ = io::stdout().flush();
}
