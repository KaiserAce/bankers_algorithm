use std::io;
use std::io::Write;

struct BankersAlgorithm {
    available: Vec<i32>,
    resources: Vec<u8>,
    processes: Vec<Process>,
}

#[derive(Debug, Clone)]
struct Process {
    id: usize,
    allocation: Vec<u8>,
    max_need: Vec<u8>,
    need: Vec<u8>,
}

fn get_numbers_from_input() -> Option<Vec<u8>> {
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("Error reading input line.");
        return None;
    }

    let numbers: Result<Vec<u8>, _> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<u8>())
        .collect();

    match numbers {
        Ok(nums) => Some(nums),
        Err(e) => {
            eprintln!("Invalid number input: {}. Please enter space-separated positive integers.", e);
            None
        }
    }
}

fn read_yes_no() -> bool {
    loop {
        print!("Create another process? [y/n]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let trimmed_input = input.trim().to_lowercase();

        match trimmed_input.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    }
}

impl Process {
    fn new(id: usize, allocation: Vec<u8>, max_need: Vec<u8>) -> Result<Process, String> {
        if allocation.len() != max_need.len() {
            return Err(format!("Process {}: Allocation and Max Need length mismatch.", id));
        }
        let mut need: Vec<u8> = Vec::with_capacity(allocation.len());
        for i in 0..allocation.len() {
            if allocation[i] > max_need[i] {
                 return Err(format!("Process {}: Allocation ({}) exceeds Max Need ({}) for resource {}.", id, allocation[i], max_need[i], i));
            }
            need.push(max_need[i] - allocation[i]);
        }
        Ok(Process {
            id,
            allocation,
            max_need,
            need,
        })
    }
}

impl BankersAlgorithm {
    fn new() -> Option<BankersAlgorithm> {
        println!("--- Banker's Algorithm Initialization ---");

        let resources = loop {
            println!("Enter resources array (e.g., 10 5 7): ");
            if let Some(res) = get_numbers_from_input() {
                if !res.is_empty() {
                    break res;
                }
            }
        };

        let num_resources = resources.len();

        let mut processes: Vec<Process> =  Vec::new();
        let mut total_allocated = vec![0u8; num_resources];

        println!("\n--- Process Creation ---");

        loop {
            let process_id = processes.len();
            println!("\n --- Enter details for P{} ---", process_id);

            let allocation = loop {
                print!("Enter current allocation for P{} ({} values):", process_id, num_resources);
                io::stdout().flush().unwrap();

                if let Some(alloc) = get_numbers_from_input() {
                    if alloc.len() == num_resources {
                        let mut possible = true;

                        for i in 0..num_resources {
                            if alloc[i] > resources[i] {
                                eprintln!("Error P{} allocation ({}) for resource {} exceeds total resources ({}).", process_id, alloc[i], i, resources[i]);
                                possible = false;
                                break;
                            }
                        }
                        if possible {break alloc;}
                    } else {
                        eprintln!("Error! Expected {} values for allocation, got {}.", num_resources, alloc.len());
                    }
                }
                println!("Try again");
            };

            let max_need = loop {
                print!("Enter maximum need for P{} ({} values): ", process_id, num_resources);
                io::stdout().flush().unwrap();

                if let Some(max) = get_numbers_from_input() {
                    if max.len() == num_resources {
                        let mut possible = true;

                        for i in 0..num_resources {
                            if max[i] > resources[i] {
                                eprintln!("Error! P{} max need({}) for resource {} exceeds total system resources ({})", process_id, max[i], i, resources[i]);
                                possible = false;
                                break;
                            }
                        }
                        
                        if possible {break max;} 

                    } else {
                        eprintln!("Error! Expected {} values for maximum need, got {}.", num_resources, max.len());
                    }
                }
                println!("Try again!.");
            };

            match Process::new(process_id, allocation.clone(), max_need) {
                Ok(process) => {
                    for i in 0..num_resources {
                        total_allocated[i] += process.allocation[i];
                    }
                    processes.push(process);
                }, 
                Err(e) => {
                    eprintln!("Error creating process P{}: {}", process_id, e);
                    println!("Please re-enter details for P{}", process_id);
                    continue;
                }
            }

            if !read_yes_no() {
                if processes.is_empty() {
                    println!("No process created. Exiting");
                    return None;
                }
                break;
            }
        }

        let mut available: Vec<i32> = Vec::with_capacity(num_resources);
        let mut possible_state = true;

        for i in 0..num_resources {
            let avail = resources[i] as i32 - total_allocated[i] as i32;
            if avail < 0 {
                eprintln!(
                "Error! Total allocated resources ({}) for resource {} exceed total available system resources ({}). Invalid initial state.",
                    total_allocated[i], i, resources[i]
            );
                possible_state = false
            }
            available.push(avail);
        }

        if !possible_state {
            println!("Cannot proceed due to invalid initial resource allocation.");
            return None;
        }

        println!("\n--- System State Initiatlized ---");
        println!("Total Resources: {:?}", resources);
        println!("Initial Available: {:?}", available);

        for p in &processes {
            println!(" P{}: Allocated={:?}, Max={:?}, Need={:?} ", p.id, p.allocation, p.max_need, p.need);
        }
        println!("-----------------------------------");

        Some(BankersAlgorithm {
                    available,
                    resources,
                    processes,
                })
    }

    fn is_safe_state(&mut self) -> Option<Vec<usize>> {
        let num_processes = self.processes.len();
        let num_resources = self.resources.len();

        let mut work: Vec<i32> = self.available.clone();
        let mut finish: Vec<bool> = vec![false; num_processes];
        let mut safe_sequence: Vec<usize> = Vec::with_capacity(num_processes);

        loop {
            let mut found_process_this_pass = false;
            for i in 0..num_processes {
                if !finish[i] {
                    let mut can_allocate = true;

                    for k in 0..num_resources {
                        if self.processes[i].need[k] as i32 > work[k] {
                            can_allocate = false;
                            break;
                        }
                    }

                    if can_allocate {
                        for k in 0..num_resources {
                            work[k] += self.processes[i].allocation[k] as i32;
                        }
                        finish[i] = true;
                        safe_sequence.push(self.processes[i].id);
                        found_process_this_pass = true;
                    }
                }
            }

            if !found_process_this_pass {
                break;
            }
        }

        if finish.iter().all(|&f| f) {
            Some(safe_sequence)
        } else {
            None
        }
    }
}

fn main() {
    if let Some(mut banker) = BankersAlgorithm::new() {
        println!("\n--- Checking System Safety ---");

        match banker.is_safe_state() {
            Some(sequence) => {
                println!("System is in a safe state.");

                let seq: Vec<String> = sequence.iter().map(|&id| format!("P{}", id)).collect();
                println!("  Safe sequence: {}", seq.join(" -> "));
            },
            None => {
                eprintln!("System is in an unsafe state! Deadlock potential exists");
            }
        }
    } else {
        println!("Initialization failed");
    }
}
