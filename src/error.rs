
static mut HAD_ERROR: bool = false;

pub fn error(line: usize, message: String) {
    report(line, "".to_string(), message);
}

fn report(line: usize, wh: String, message: String) {
    eprintln!("[line {}] Error{}: {}", line, wh, message);
    unsafe { 
        HAD_ERROR = true; 
    }
}

pub fn failed() -> bool {
    unsafe {
        HAD_ERROR
    }
}