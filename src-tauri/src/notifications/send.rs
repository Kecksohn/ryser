use super::notification_manager;
use super::Result;

use super::notification_manager::send_msg;

pub(crate) fn show_msg_gui(
    header: &str, 
    message: &str,
    message_id: Option<String>,
    duration_ms: Option<u64>
) -> Result<String> {
    send_msg(header, message, message_id, None)
}