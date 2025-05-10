use itertools::Itertools;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Params {
    proc_times: Vec<ProcessTime>,
    count_procs: usize,
    count_mac: usize,
}

type Machine = u8;
type ProcessTime = u32;

#[derive(Debug, Clone)]
pub struct Solution {
    // Vetor que diz qual máquina cada processo está alocado
    tasks: Vec<Machine>,
    largest_machine_time: ProcessTime,
}

impl Solution {
    fn new(tasks: Vec<Machine>, params: &Params) -> Self {
        let mut this = Self {
            tasks,
            largest_machine_time: 0,
        };
        this.largest_machine_time = this.largest_machine_time(params);
        return this;
    }
    fn neighbour_for_task(&self, task: usize, params: &Params) -> Vec<Self> {
        let now = self.tasks[task];
        (0..(params.count_mac as Machine))
            .filter(|mac| *mac != now)
            .map(|new_mac| {
                let mut new = self.clone();
                new.tasks[task] = new_mac;
                new
            })
            .collect_vec()
    }

    fn time_by_machine(&self, params: &Params) -> Vec<Vec<(usize, ProcessTime)>> {
        let mut times = vec![Vec::new(); params.count_mac];
        for (i, mac) in self.tasks.iter().enumerate() {
            times[*mac as usize].push((i, params.proc_times[i]));
        }
        times
    }
    fn largest_machine_time(&self, params: &Params) -> ProcessTime {
        self.time_by_machine(params)
            .into_iter()
            .map(|bars| bars.into_iter().map(|(_, time)| time).sum())
            .max()
            .expect("Nenhuma máquina")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PParams {
    pub i_max: usize,
    pub idle_max: usize,
    pub a: f64,
}

// g é a melhora em relação a solução atual
fn eval_candidate(s: &Solution, s_prime: &Solution) -> ProcessTime {
    s.largest_machine_time - s_prime.largest_machine_time
}

fn random_greedy_solution(params: &Params, a: f64, rand: &mut impl rand::Rng) -> Solution {
    // solução inicial vazia
    let tasks = std::iter::from_fn(|| Some(rand.gen_range(0..params.count_mac as Machine)))
        .take(params.count_procs)
        .collect_vec();
    let mut s = Solution::new(tasks, params);

    let mut lc = params.count_procs * params.count_mac;

    while lc != 0 {
        // Cria a lrc com candidatos e seus g's
        let mut lrc = (0..params.count_procs)
            .into_iter()
            .map(|task| s.neighbour_for_task(task, params))
            .flatten()
            .map(|s_prime| {
                let g = eval_candidate(&s, &s_prime);
                (s_prime, g)
            })
            .collect_vec();
        // Nenhum vizinho é válido
        if lrc.is_empty() {
            break;
        }
        // ordena por valor de g
        lrc.sort_unstable_by_key(|k| k.1);
        let worst = lrc[0].1;
        let best = lrc[lrc.len() - 1].1;
        let cutoff = a * ((best - worst) + worst) as f64;

        // Remove candidatos cujo g não atende os parametros
        // nesse caso é >= pois é um problema de max
        lrc.retain(|(_, weight)| *weight as f64 >= cutoff);

        // escolhe candidato aleatorialemente
        let (flipped, _) = lrc.choose_mut(rand).unwrap();
        // remove candidato da LC
        lc -= 1;

        // s = flipped;
        std::mem::swap(&mut s, flipped);
    }
    s
}

fn greedy_search(s: &mut Solution, params: &Params) {
    let mut times = s.time_by_machine(params);
    times.retain(|bar| !bar.is_empty());
    times.sort_by_cached_key(|bar| bar.iter().map(|(_, val)| val).sum::<ProcessTime>());
    for bar in &mut times {
        bar.sort_by_cached_key(|(_, time)| *time);
    }
    let menor = &times[0][0];
    let maior = &times[times.len() - 1];
    let maior = maior[maior.len() - 1];
    let mut tasks2 = s.tasks.clone();
    tasks2.swap(menor.0, maior.0);
    let s_prime = Solution::new(tasks2, params);
    if s_prime.largest_machine_time < s.largest_machine_time {
        *s = s_prime;
        greedy_search(s, params);
    }
}

fn run(pparams: PParams, params: &Params) -> (Duration, Solution, Vec<Solution>) {
    let PParams {
        i_max, a, idle_max, ..
    } = pparams;

    let mut s_best = None;

    let mut improvements = vec![];
    let mut rand = rand::thread_rng();

    let mut idle = 0;

    let now = Instant::now();
    for i in 0.. {
        let mut s = random_greedy_solution(params, a, &mut rand);
        greedy_search(&mut s, params);

        // Na primeira iteração não há uma solução melhor ainda
        let Some(ref mut s_best) = s_best else {
            // Então, se estivermos na primeira iteração, seu s será o best.
            improvements.push(s.clone());
            s_best = Some(s);

            continue;
        };

        if s_best.largest_machine_time > s.largest_machine_time {
            improvements.push(s.clone());
            *s_best = s;
            idle = 0;
        } else {
            idle += 1;
        }

        // se idle_max != 0, quer dizer que estamos limitando por iterações sem melhoria
        if idle_max != 0 && idle >= idle_max {
            // quantidade de turnos sem melhora excedeu o parâmetro.
            break;
        }

        // se i_max != 0, quer dizer que estamos limitando por quantidade de iterações
        if i_max != 0 && i >= i_max {
            // quantidade de iterações excedeu o máximo.
            break;
        }
    }
    let runtime = now.elapsed();

    (runtime, s_best.unwrap(), improvements)
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let nome = std::env::args().nth(1).unwrap_or("facil.txt".into());
    let arquivo = std::fs::read_to_string(nome)?;
    let mut tempo_tasks: Vec<ProcessTime> = vec![];
    let linhas = arquivo.split("\n");
    let mut reading_machines = true;
    for linha in linhas {
        if linha.is_empty() {
            continue;
        }
        tempo_tasks.push(linha.parse()?);
    }

    let pparams = PParams {
        i_max: 0,
        idle_max: 300,
        a: 0.5,
    };

    let params = Params {
        count_procs: tempo_tasks.len(),
        count_mac: 5,
        proc_times: tempo_tasks,
    };

    for _ in 0..10 {
        let (runtime, solution, _) = run(pparams, &params);
        println!("[{:?}] {:?}", runtime, solution.largest_machine_time);
    }
    Ok(())
}
