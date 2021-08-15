pub mod events;
pub mod sharedbuffer;

use bash_builtins::error;
use std::mem::MaybeUninit;
use std::sync::Once;
use std::time::Duration;

pub use sharedbuffer::{SharedBuffer, SharedBufferGuard};

/// Size for the shared buffer;
const SHARED_BUFFER_SIZE: usize = 8 * 1024;

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
