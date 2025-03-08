use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::thread::{self, Thread};

use ahash::HashMap;
use nu_plugin::EngineInterface;
use nu_protocol::{ShellError, Spanned, engine::Closure};

use crate::Task;
use crate::utils::collections::SlotVec;

#[derive(Debug)]
pub struct Scheduler<'a> {
    tasks: Vec<&'a Task>,
    adj_list: Vec<Vec<TaskId>>,
    in_degrees: Vec<usize>,
    jobs: usize,
}

impl<'a> Scheduler<'a> {
    pub fn new(jobs: Option<NonZeroUsize>) -> Self {
        Self {
            tasks: vec![],
            adj_list: vec![],
            in_degrees: vec![],
            jobs: jobs.map(NonZeroUsize::get).unwrap_or_else(num_cpus::get),
        }
    }

    pub fn add_task(&mut self, task: &'a Task, deps: &[TaskId]) -> TaskId {
        let id = TaskId(self.tasks.len());
        self.tasks.push(task);
        self.adj_list.push(Vec::new());
        self.in_degrees.push(deps.len());
        for &TaskId(dep) in deps {
            self.adj_list[dep].push(id);
        }
        id
    }

    pub fn run(mut self, engine: &EngineInterface) -> Result<(), ShellError> {
        let mut run_queue = VecDeque::new();
        let mut maybe_skip_queue = VecDeque::new();

        for (id, &in_degree) in self.in_degrees.iter().enumerate() {
            if in_degree == 0 {
                if let Some(timestamp) = self.tasks[id].cached_at() {
                    maybe_skip_queue.push_front((TaskId(id), timestamp));
                } else {
                    run_queue.push_front(TaskId(id));
                }
            }
        }

        let mut latest_dep_timestamp = HashMap::default();
        while let Some((TaskId(dep_id), dep_timestamp)) = maybe_skip_queue.pop_back() {
            for &TaskId(id) in &self.adj_list[dep_id] {
                self.in_degrees[id] -= 1;
                let t = latest_dep_timestamp.entry(id).or_insert(dep_timestamp);
                *t = (*t).max(dep_timestamp);
                if self.in_degrees[id] == 0 {
                    match self.tasks[id].cached_at() {
                        Some(timestamp) if latest_dep_timestamp[&id] <= timestamp => {
                            maybe_skip_queue.push_back((TaskId(id), timestamp));
                        }
                        _ => run_queue.push_front(TaskId(id)),
                    }
                }
            }
        }

        let thread_token = thread::current();
        thread::scope(|sc| -> Result<(), ShellError> {
            let mut free_slots = self.jobs;
            let mut run_set = SlotVec::default();

            loop {
                while free_slots > 0 && !run_queue.is_empty() {
                    let TaskId(id) = run_queue.pop_back().unwrap();
                    let prompt = self.tasks[id].name();
                    let run = self.tasks[id].run();

                    struct Capture<'scope> {
                        id: usize,
                        prompt: &'scope str,
                        run: Option<&'scope Spanned<Closure>>,
                        thread_token: &'scope Thread,
                    }

                    let stask = Capture {
                        id,
                        prompt,
                        run,
                        thread_token: &thread_token,
                    };

                    run_set.insert(sc.spawn(move || -> Result<usize, ShellError> {
                        let Capture {
                            id,
                            prompt,
                            run,
                            thread_token,
                        } = stask;

                        println!("Running task `{prompt}`");
                        if let Some(run) = run {
                            engine
                                .eval_closure(run, vec![], None)
                                .inspect_err(|_| thread_token.unpark())?;
                        }
                        thread_token.unpark();
                        Ok(id)
                    }));
                    free_slots -= 1;
                }

                if !run_set.is_empty() {
                    thread::park();
                } else if run_queue.is_empty() {
                    break Ok(());
                }

                for res in run_set
                    .drain(|task| task.is_finished())
                    .map(|task| task.join().unwrap())
                {
                    let id = res?;
                    free_slots += 1;

                    for &TaskId(next) in &self.adj_list[id] {
                        self.in_degrees[next] -= 1;
                        if self.in_degrees[next] == 0 {
                            run_queue.push_front(TaskId(next));
                        }
                    }
                }
            }
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(usize);
