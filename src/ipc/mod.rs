pub mod events;
pub mod sharedbuffer;

use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::sync::Once;
use std::time::Duration;

use bash_builtins::{error, variables::DynamicVariable};

pub use sharedbuffer::{SharedBuffer, SharedBufferGuard};

/// Size for the shared buffer;
const SHARED_BUFFER_SIZE: usize = 16 * 1024;

/// Timeout to access the inner value of `max_cmdline` from the
/// `TIMEHISTORY_CMDLINE_LIMIT` variable.
const TIMEOUT_CMDLINE_VAR: Duration = Duration::from_secs(1);

/// Global reference to the shared buffer.
pub fn global_shared_buffer(timeout: Duration) -> Option<SharedBufferGuard<'static>> {
    static mut BUFFER: MaybeUninit<Option<SharedBuffer>> = MaybeUninit::uninit();
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let sb = match SharedBuffer::new(SHARED_BUFFER_SIZE) {
            Ok(sb) => Some(sb),

            Err(e) => {
                error!("failed to initialize shared buffer: {}", e);
                None
            }
        };

        unsafe {
            BUFFER = MaybeUninit::new(sb);
        }
    });

    let buffer = unsafe { (&*BUFFER.as_ptr()).as_ref() };
    buffer.and_then(|b| b.lock(timeout).ok())
}

/// Dynamic variable to control the cmdline limit.
pub struct CmdLineLimitVariable;

impl DynamicVariable for CmdLineLimitVariable {
    fn get(&mut self) -> std::option::Option<CString> {
        let max_cmdline = global_shared_buffer(TIMEOUT_CMDLINE_VAR)?.max_cmdline();

        CString::new(max_cmdline.to_string()).ok()
    }

    fn set(&mut self, value: &CStr) {
        let max_cmdline = match value.to_str().map(str::parse) {
            Ok(Ok(n)) => n,

            _ => {
                let _ = writeln!(io::stderr(), "timehistory: invalid number");
                return;
            }
        };

        if let Some(mut buffer) = global_shared_buffer(TIMEOUT_CMDLINE_VAR) {
            buffer.set_max_cmdline(max_cmdline);
        }
    }
}
