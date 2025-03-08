use ahash::HashMap;
use nu_protocol::{LabeledError, Spanned};

use crate::{Scheduler, Task, TaskId};

#[derive(Debug)]
pub struct TaskGraph<'t, 's> {
    name2task: HashMap<&'t str, &'t Task>,
    name2id: HashMap<&'t str, Option<TaskId>>,
    sched: &'s mut Scheduler<'t>,
}

impl<'t, 's> TaskGraph<'t, 's>
where
    't: 's,
{
    pub fn new(tasks: impl Iterator<Item = &'t Task>, sched: &'s mut Scheduler<'t>) -> Self {
        Self {
            name2task: tasks.map(|task| (task.name(), task)).collect(),
            name2id: Default::default(),
            sched,
        }
    }

    pub fn submit(&mut self, task: &'t Task) -> Result<(), LabeledError> {
        self.submit_impl(task.name(), task)?;
        Ok(())
    }
}

impl<'t> TaskGraph<'t, '_> {
    fn submit_impl(&mut self, name: &'t str, task: &'t Task) -> Result<TaskId, SubmitError> {
        match self.name2id.get(name) {
            Some(Some(id)) => return Ok(*id),
            Some(None) => return Err(SubmitError::FoundCircularDep),
            None => (),
        }

        self.name2id.insert(name, None);

        let mut deps = vec![];
        for dname in task.deps() {
            let dep_task = self
                .name2task
                .get(dname.item.as_str())
                .copied()
                .ok_or_else(|| SubmitError::TaskNotFound {
                    name: dname.clone(),
                })?;
            let dep_id = self.submit_impl(&dname.item, dep_task).map_err(|e| {
                if let SubmitError::FoundCircularDep = e {
                    SubmitError::CircularDep {
                        parsing: dep_task.name_span().map(Into::into),
                        dep: self.name2task[name].name_span().map(Into::into),
                    }
                } else {
                    e
                }
            })?;
            deps.push(dep_id);
        }

        let id = self.sched.add_task(task, &deps);
        self.name2id.insert(name, Some(id));
        Ok(id)
    }
}

#[derive(Debug)]
enum SubmitError {
    TaskNotFound {
        name: Spanned<String>,
    },
    CircularDep {
        parsing: Spanned<String>,
        dep: Spanned<String>,
    },
    FoundCircularDep,
}

impl From<SubmitError> for LabeledError {
    fn from(e: SubmitError) -> Self {
        match e {
            SubmitError::TaskNotFound { name } => {
                LabeledError::new("Task not found").with_label("Task not found", name.span)
            }
            SubmitError::CircularDep { parsing, dep } => {
                LabeledError::new("Circular dependency between tasks")
                    .with_label("When parsing task here...", parsing.span)
                    .with_label("...it provides the task", dep.span)
            }
            SubmitError::FoundCircularDep => unreachable!(),
        }
    }
}
