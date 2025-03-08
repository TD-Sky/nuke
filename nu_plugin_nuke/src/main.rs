mod graph;
mod plugin;
mod sched;
mod task;
mod utils;

use nu_plugin::{MsgPackSerializer, serve_plugin};

pub use graph::TaskGraph;
pub use plugin::NukePlugin;
pub use sched::{Scheduler, TaskId};
pub use task::Task;

fn main() {
    serve_plugin(&NukePlugin::default(), MsgPackSerializer)
}
