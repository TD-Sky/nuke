use std::num::NonZero;
use std::path::Path;
use std::sync::{Arc, Mutex};

use glob::glob;
use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand, SimplePluginCommand};
use nu_protocol::{
    FromValue, LabeledError, ShellError, Signature, Spanned, SyntaxShape, Type, Value,
};

use crate::{Scheduler, Task, TaskGraph};

#[derive(Debug, Default)]
pub struct NukePlugin {
    tasks: Arc<boxcar::Vec<Task>>,
    entry: Mutex<Option<String>>,
}

impl Plugin for NukePlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(NukeSchedule),
            Box::new(NukeTask),
            Box::new(NukeEntry),
        ]
    }
}

#[derive(Debug)]
struct NukeTask;

impl SimplePluginCommand for NukeTask {
    type Plugin = NukePlugin;

    fn name(&self) -> &str {
        "nuke task"
    }

    fn description(&self) -> &str {
        "define task"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::Nothing, Type::record())
            .required("name", SyntaxShape::String, "task name")
            .named(
                "deps",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "the needed tasks to run first",
                None,
            )
            .named("run", SyntaxShape::Closure(None), "task body", None)
            .named("target", SyntaxShape::Filepath, "the built file", None)
            .named(
                "files",
                SyntaxShape::List(Box::new(SyntaxShape::OneOf(vec![
                    SyntaxShape::GlobPattern,
                    SyntaxShape::Filepath,
                ]))),
                "the dependent files",
                None,
            )
    }

    fn run(
        &self,
        plugin: &NukePlugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let workdir = engine.get_current_dir()?;
        let workdir = Path::new(&workdir);

        let name = Spanned::<String>::from_value(call.positional[0].clone())?;
        let deps = call.get_flag("deps")?.unwrap_or_default();
        let files = call
            .get_flag_value("files")
            .map(|vs| {
                let mut files = vec![];

                for v in vs.into_list()? {
                    match v {
                        Value::String { val, .. } => files.push(workdir.join(val)),
                        Value::Glob {
                            val, internal_span, ..
                        } => glob(&workdir.join(val).to_string_lossy())
                            .map_err(|e| ShellError::InvalidGlobPattern {
                                msg: e.msg.into(),
                                span: internal_span,
                            })?
                            .flatten()
                            .filter(|p| p.is_file())
                            .for_each(|p| files.push(p)),
                        _ => {
                            return Err(ShellError::TypeMismatch {
                                err_message: "<file> can only be `string` or `glob`".into(),
                                span: v.span(),
                            });
                        }
                    };
                }

                Result::<_, ShellError>::Ok(files)
            })
            .transpose()?
            .unwrap_or_default();
        let run = call.get_flag("run")?;
        let target = call.get_flag("target")?;

        plugin.tasks.push(Task {
            name,
            deps,
            files,
            target,
            run,
        });

        Ok(Value::nothing(call.head))
    }
}

#[derive(Debug)]
struct NukeSchedule;

impl SimplePluginCommand for NukeSchedule {
    type Plugin = NukePlugin;

    fn name(&self) -> &str {
        "nuke schedule"
    }

    fn description(&self) -> &str {
        "schedule tasks"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .optional("name", SyntaxShape::String, "task name")
            .named(
                "jobs",
                SyntaxShape::Int,
                "Number of parallel jobs [default: the number of cores]",
                Some('j'),
            )
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let jobs = call
            .get_flag::<usize>("jobs")?
            .map(|n| {
                NonZero::new(n).ok_or_else(|| LabeledError::new("<jobs> should be greater than 0"))
            })
            .transpose()?;

        let task = {
            let entry = plugin.entry.lock().unwrap();
            let name = if let Some(name) = call.positional.first() {
                name.as_str()?
            } else if let Some(name) = entry.as_ref() {
                name.as_str()
            } else {
                return Err(LabeledError::new("Missing task to run")
                    .with_help("Nuke expects a task to start. \
                        Pass it when running `nuke schedule <name>` or mark it with `nuke entry <name>`"));
            };

            plugin
                .tasks
                .iter()
                .find_map(|(_, task)| (task.name() == name).then_some(task))
                .ok_or_else(|| LabeledError::new(format!("task `{name}` not found")))?
        };

        let mut sched = Scheduler::new(jobs);

        TaskGraph::new(plugin.tasks.iter().map(|(_, task)| task), &mut sched).submit(task)?;

        sched.run(engine)?;

        Ok(Value::nothing(call.head))
    }
}

#[derive(Debug)]
struct NukeEntry;

impl SimplePluginCommand for NukeEntry {
    type Plugin = NukePlugin;

    fn name(&self) -> &str {
        "nuke entry"
    }

    fn description(&self) -> &str {
        "Set entry task"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self)).required(
            "name",
            SyntaxShape::String,
            "task name",
        )
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        *plugin.entry.lock().unwrap() = Some(call.positional[0].as_str()?.into());

        Ok(Value::nothing(call.head))
    }
}
