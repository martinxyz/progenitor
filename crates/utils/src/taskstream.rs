use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc,
};

/// Run a stream of tasks in parallel, with dependencies.
///
/// process_task() is called from a thread pool (arbitrary order)
///
/// process_result() is called synchronously, in the same order that tasks were
/// submitted. It may return a new task to submit.
pub fn run_stream<T, R, FT, FR>(
    initial_tasks: impl IntoIterator<Item = T>,
    process_task: FT,
    mut process_result: FR,
) where
    T: Send,
    R: Send,
    FT: Fn(T) -> R + Send + Copy,
    FR: FnMut((usize, R)) -> Option<T>,
{
    rayon::in_place_scope(|s| {
        // Without this limit, rayon tends to leave some old tasks unfinished,
        // making us store more inputs/outputs. Previously this value affected
        // performance, but now (since using mpsc) it seems not to matter much.
        let tasks_spawned_max = num_cpus::get() * 2;

        let mut tasks: VecDeque<T> = initial_tasks.into_iter().collect();
        let mut results = HashMap::<usize, R>::new();
        let (channel_s, channel_r) = mpsc::channel();

        let mut task_i = 0;
        let mut result_i = 0;
        let mut tasks_spawned = 0;
        while result_i < task_i || !tasks.is_empty() {
            while !results.contains_key(&result_i) {
                // spawn tasks
                while tasks_spawned < tasks_spawned_max {
                    match tasks.pop_front() {
                        Some(task) => {
                            let channel_s = channel_s.clone();
                            s.spawn(move |_| {
                                channel_s.send((task_i, process_task(task))).unwrap();
                            });
                            tasks_spawned += 1;
                            task_i += 1;
                        }
                        _ => {
                            break;
                        }
                    }
                }
                // wait for a result
                let (i, result) = channel_r.recv().unwrap();
                results.insert(i, result);
                tasks_spawned -= 1;
            }
            let result = results.remove(&result_i).unwrap();
            result_i += 1;
            // process result
            if let Some(new_task) = process_result((result_i, result)) {
                tasks.push_back(new_task);
            }
        }
        assert_eq!(tasks_spawned, 0);
        assert_eq!(task_i, result_i);
        assert!(results.is_empty());
    });
}
