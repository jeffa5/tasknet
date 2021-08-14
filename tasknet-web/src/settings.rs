use serde::{Deserialize, Serialize};

use crate::task::{Priority, Status, Task};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Settings {}

impl Default for Settings {
    fn default() -> Self {
        Self {}
    }
}
