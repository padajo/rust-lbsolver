use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env::args_os;
use std::fs::File;
use std::io::{self, BufRead};
// use std::sync::WaitTimeoutResult;

#[derive(PartialEq, Eq)]
struct State {
    chain: Vec<String>,
    last_word: String,
    last_char: char,
    cost: usize,
    heuristic: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct LetterBoxedSolver {
    // letter_groups: Vec<Vec<char>>,
    available_chars: HashSet<char>,
    dictionary: Vec<String>,
    start_letter_dictionary: HashMap<char, Vec<String>>,
    // end_letter_dictionary: HashMap<char, Vec<String>>,
}

impl LetterBoxedSolver {
    fn new(string_groups: &Vec<String>, source: File) -> LetterBoxedSolver {
        let mut dictionary: Vec<String> = Vec::new();
        let mut letter_groups: Vec<Vec<char>> = Vec::new();
        let mut available_chars: HashSet<char> = HashSet::new();
        let mut start_letter_dictionary: HashMap<char, Vec<String>> = HashMap::new();
        let mut end_letter_dictionary: HashMap<char, Vec<String>> = HashMap::new();

        for group in string_groups {
            let mut chars: Vec<char> = group.chars().collect();
            if chars.len() != 3 {
                panic!("Each group of letters must be 3 letters long");
            }
            chars.sort();
            letter_groups.push(chars.clone());
            available_chars.extend(chars.clone());
        }

        // check if there are duplicate letters in the available chars
        let no_duplicate_letters = available_chars.len() == 12;

        let lines = io::BufReader::new(source).lines();
        for line in lines {
            if let Ok(word) = line {
                // we can't use words longer than 12 letters or shorter than 3 letters
                if word.len() > 12 || word.len() < 3 {
                    continue;
                }
                // only push if the word has letters that are all in the available chars
                let word_chars: HashSet<char> = word.chars().collect();

                // if no duplicates in the letter groups then
                // check if there are duplicate letters in the word and reject if there are
                if no_duplicate_letters && (word_chars.len() != word.len()) {
                    continue;
                }

                let diff: HashSet<_> = word_chars.difference(&available_chars).collect();
                // if there is any difference (length > 0), then don't add this word to the dictionary
                if diff.len() > 0 {
                    continue;
                }

                // this works out if the words in the dictionary are valid
                // for these letter groups
                let is_letter_box_word =
                    LetterBoxedSolver::is_letter_pattern_in_letter_box(&letter_groups, &word);
                if !is_letter_box_word {
                    continue;
                }

                dictionary.push(word.clone());
                let start_letter = word.chars().next().unwrap();
                let end_letter = word.chars().last().unwrap();

                // add to start letter dictionary
                if start_letter_dictionary.contains_key(&start_letter) {
                    let words = start_letter_dictionary.get_mut(&start_letter).unwrap();
                    words.push(word.clone());
                } else {
                    start_letter_dictionary.insert(start_letter, vec![word.clone()]);
                }

                // add to end letter dictionary
                if end_letter_dictionary.contains_key(&end_letter) {
                    let words = end_letter_dictionary.get_mut(&end_letter).unwrap();
                    words.push(word.clone());
                } else {
                    end_letter_dictionary.insert(end_letter, vec![word.clone()]);
                }
            }
        }

        // reorder dictionary by word length, longest first
        dictionary.sort_by(|a, b| b.len().cmp(&a.len()));

        LetterBoxedSolver {
            // letter_groups,
            available_chars,
            dictionary,
            start_letter_dictionary,
            // end_letter_dictionary,
        }
    }

    // this is the solver part of the program
    fn run_solver(&mut self, ignore_words: &Vec<String>) -> Result<Vec<Vec<String>>, String> {
        // for c in self.available_chars.clone() {
        //     let mut words = self.start_letter_dictionary.get_mut(&c).unwrap();
        //     println!("{}: {:?}", c, words.len());
        // }

        return Ok(self.a_star(ignore_words).unwrap());
    }

    // useful for A* search
    fn heuristic(&self, chain: &Vec<String>, chars: &HashSet<char>) -> usize {
        let chain_chars: HashSet<char> = chain.iter().flat_map(|word| word.chars()).collect();
        let diff = chars.difference(&chain_chars).count();

        let mut a = chars.into_iter().collect::<Vec<_>>();
        let mut b = chain_chars.into_iter().collect::<Vec<_>>();
        a.sort();
        b.sort();

        // println!("Chars: {}", a.into_iter().collect::<String>());
        // println!("------------------------------");
        // println!("Chain: {:?}", chain);
        // println!("ch-ch: {}", b.into_iter().collect::<String>());
        // println!(" diff: {:?}", diff);
        // println!("\n---\n");

        diff
    }

    // this works, and returns quite fast
    fn a_star(&mut self, ignore_words: &Vec<String>) -> Option<Vec<Vec<String>>> {
        let graph: &HashMap<char, Vec<String>> = &self.start_letter_dictionary;

        let mut priority_queue = BinaryHeap::new();

        let mut solutions: Vec<Vec<String>> = Vec::new();
        // let mut max_solution_length = 1;

        // println!("Avaiable chars: {:?}", self.available_chars);

        // reset priority queue every loop
        for word in &self.dictionary {
            // ignore this word if it's in the ignore_words list
            if ignore_words.contains(word) {
                continue;
            }
            let last_char = word.chars().last().unwrap();
            priority_queue.push(State {
                chain: vec![word.clone()],
                last_word: word.clone(),
                last_char,
                cost: 1,
                heuristic: self.heuristic(&vec![word.clone()], &self.available_chars),
            });
        }

        let mut visited = BinaryHeap::new();

        let return_after = 4;

        // find a solution with 1 word, then 2, then 3 etc
        // this will find the shortest solution
        for l in 1..=6 {
            // reset solutions and max solution length
            solutions = Vec::new();
            let max_solution_length = l;
            // priority_queue = BinaryHeap::new();

            while let Some(state) = priority_queue.pop() {
                // add to visited
                visited.push(State {
                    chain: state.chain.clone(),
                    last_word: state.last_word.clone(),
                    last_char: state.last_char,
                    cost: state.cost,
                    heuristic: state.heuristic,
                });

                // if it's too long by more than 1, skip it
                if state.chain.len() > max_solution_length {
                    continue;
                }
                if state.heuristic == 0 {
                    // return Some(state.chain);
                    solutions.push(state.chain.clone());
                    // println!("Solution found: {:?}", state.chain.clone());
                    // just return the first one found if we're on 4 words
                    // or if we've got 20 solutions, return those
                    if max_solution_length > 3 || solutions.len() >= return_after {
                        return Some(solutions);
                    }
                }

                // println!("Chain: {:?}", state.chain);

                if let Some(next_words) = graph.get(&state.last_char) {
                    for next_word in next_words {
                        // don't add the word if it's already in the chain
                        if state.chain.contains(next_word) {
                            continue;
                        }
                        // if the word is in the ignore_words list, skip it
                        if ignore_words.contains(next_word) {
                            continue;
                        }

                        // println!("\tWord: {}", next_word);

                        let mut new_chain = state.chain.clone();
                        new_chain.push(next_word.clone());

                        let h = self.heuristic(&new_chain, &self.available_chars);

                        let last_char = next_word.chars().last().unwrap();
                        priority_queue.push(State {
                            chain: new_chain.clone(),
                            last_word: next_word.clone(),
                            last_char,
                            cost: state.cost + 1,
                            heuristic: h,
                        });
                    }
                }
            }

            if solutions.len() > 0 {
                // println!("Found {} solutions", solutions.len());
                // println!("Solutions: {:?}", solutions);
                break;
            } else {
                println!("No solutions found with {} words in the chain", l);
                // println!("Moving visited to priority queue");
                // let mut i = 0;
                while let Some(state) = visited.pop() {
                    priority_queue.push(state);
                    // i += 1;
                }
                // println!("{} states moved", i);
            }
        }

        Some(solutions)
    }

    // this is recursive
    // take the first two letters, and check they are in different
    // groups, then if there are any letters left, remove the first
    // letter and recurse
    fn is_letter_pattern_in_letter_box(letter_groups: &Vec<Vec<char>>, word: &String) -> bool {
        // get first two characters of word
        let chars: Vec<char> = word.chars().collect();
        for i in 0..letter_groups.len() {
            let group = &letter_groups[i];
            if group.contains(&chars[0]) && group.contains(&chars[1]) {
                // if we get here then the first two letters are in the same group
                // which won't work
                return false;
            }
        }
        // if we get here then the first two letters are in different groups
        // get all letters of word minus the first letter
        let remaining_chars: Vec<char> = chars[1..].to_vec();
        // we can't check a word with less than 2 characters
        if remaining_chars.len() == 1 {
            return true;
        }

        // send a string of the remaining characters to the function
        // if it returns true then we have a match
        return LetterBoxedSolver::is_letter_pattern_in_letter_box(
            letter_groups,
            &remaining_chars.iter().collect(),
        );
    }
}

fn main() {
    println!("Starting Letter Boxed Solver...");

    let filename: String = "./yawl_mendel_lee_cooper_word-list-for-lb.txt".to_string();
    let file = File::open(filename);

    let args_count: usize = args_os().count();

    if args_count < 5 {
        println!("Usage: lbsolver <group1> <group2> <group3> <group4> <ignore_word (opt)> <ignore_word (opt)> ...");
        println!("Each group must be 3 letters long");
        println!("Any words after the 4 groups of 3 letters will be filtered out in the searching");
        return;
    }

    let args = args_os().into_iter().collect::<Vec<_>>();

    // turn args into Vec<String>
    let args_string: Vec<String> = args
        .iter()
        .map(|arg| arg.to_owned().into_string().unwrap())
        .collect();

    let groups: Vec<String> = args_string[1..5].to_vec();

    let ignore_words: Vec<String> = args_string[5..].to_vec();

    let mut solver = LetterBoxedSolver::new(&groups, file.unwrap());

    let found_solutions = solver.run_solver(&ignore_words);
    println!("Groups: {:?}", groups);
    println!("Ignore: {:?}", ignore_words);

    if found_solutions.is_ok() {
        let solutions = found_solutions.unwrap();

        println!("\n{} solutions found\n", solutions.len());

        for solution in solutions {
            println!("Solution: {:?}", solution);
        }
    } else {
        println!("No solution found");
    }
}
