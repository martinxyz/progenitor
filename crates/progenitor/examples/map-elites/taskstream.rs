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
        let max_pending_tasks = num_cpus::get() * 2;

        let mut tasks: VecDeque<(usize, T)> = initial_tasks.into_iter().enumerate().collect();
        let mut total_tasks_queued = tasks.len();
        let (results_s, results_r) = mpsc::channel();

        let mut results = HashMap::<usize, R>::new();
        let mut tasks_pending = 0;

        // Ugh. Simplify loop conditions?
        let mut i = 0;
        while !tasks.is_empty() || !results.is_empty() || tasks_pending > 0 {
            while !results.contains_key(&i) {
                // schedule more tasks
                while tasks_pending < max_pending_tasks && !tasks.is_empty() {
                    tasks_pending += 1;
                    let results_s = results_s.clone();
                    let (j, params) = tasks.pop_front().unwrap();
                    s.spawn(move |_| {
                        results_s.send((j, process_task(params))).unwrap();
                    });
                }
                // pop one result
                let (j, res) = results_r.recv().unwrap();
                tasks_pending -= 1;
                results.insert(j, res);
            }
            let result = results.remove(&i).unwrap();

            if let Some(new_task) = process_result((i, result)) {
                tasks.push_back((total_tasks_queued, new_task));
                total_tasks_queued += 1;
            }
            i += 1;
        }
    });
}
