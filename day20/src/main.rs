use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::mem;

/*
Notes:

% = flip-flop, stores a state. Receiving a low pulse means "switch state".
& = an and.
broadcaster = just resends pulse
button = when it's pressed, sends a low pulse to the broadcaster
*/

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Pulse {
    High,
    Low,
}
use Pulse::*;

#[derive(Debug, Clone, Copy)]
enum OnOrOff {
    On,
    Off,
}
use OnOrOff::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
struct ModuleKey(String);

#[derive(Debug)]
enum ModuleType {
    FlipFlop {
        internal_state: OnOrOff,
    },
    Nand {
        input_component_to_last_known_pulse: HashMap<ModuleKey, Pulse>,
        number_of_low_pulses: usize,
    },
    Broadcaster,
    Sink,
}
use ModuleType::*;

impl ModuleType {
    fn to_compact_state(&self) -> usize {
        match self {
            FlipFlop { internal_state } => match internal_state {
                Off => 0,
                On => 1,
            },
            Nand {
                input_component_to_last_known_pulse,
                number_of_low_pulses: _,
            } => {
                let mut keys: Vec<ModuleKey> = input_component_to_last_known_pulse
                    .keys()
                    .cloned()
                    .collect();
                keys.sort();

                let mut total = 0;
                for (i, key) in keys.iter().enumerate() {
                    let pulse = input_component_to_last_known_pulse.get(key).unwrap();
                    total += (2usize.pow(i as u32))
                        * (match pulse {
                            High => 1,
                            Low => 0,
                        })
                }
                total
            }
            Broadcaster => 0,
            Sink => 0,
        }
    }

    fn process_pulse(&mut self, module_key: &ModuleKey, pulse: &Pulse) -> Option<Pulse> {
        match self {
            FlipFlop { internal_state } => match pulse {
                /* Flip-flop - this ignores high pulses, and makes low pulses switch the internal state. */
                High => None,
                Low => {
                    let new_internal_state = match internal_state {
                        On => Off,
                        Off => On,
                    };
                    let _ = mem::replace(internal_state, new_internal_state);
                    match new_internal_state {
                        On => Some(High),
                        Off => Some(Low),
                    }
                }
            },
            Nand {
                input_component_to_last_known_pulse,
                number_of_low_pulses,
            } => {
                /* This is essentially a NAND gate - remembers the old pulse from all of its inputs. */
                let old_pulse = input_component_to_last_known_pulse.get(module_key).expect("BUG - didn't previously see a value from this component, should have been initialised to empty");
                if old_pulse != pulse {
                    // Update our internal state.
                    let _old_state =
                        input_component_to_last_known_pulse.insert(module_key.clone(), *pulse);
                    match pulse {
                        High => *number_of_low_pulses -= 1,
                        Low => *number_of_low_pulses += 1,
                    };
                }

                if *number_of_low_pulses == 0 {
                    Some(Low)
                } else {
                    Some(High)
                }
            }
            Broadcaster => Some(*pulse),
            Sink => None,
        }
    }
}

#[derive(Debug)]
struct Module {
    type_: ModuleType,
    outputs: Vec<ModuleKey>,
}

#[derive(Debug)]
struct Modules {
    modules: HashMap<ModuleKey, Module>,
    // (module0, module1, pulse) means that module0 sent pulse to module1.
    pulse_queue: VecDeque<(ModuleKey, ModuleKey, Pulse)>,
    total_high_pulses: usize,
    total_low_pulses: usize,
}

impl Modules {
    fn compact_state(&self) -> String {
        assert!(self.pulse_queue.is_empty());

        self.modules
            .iter()
            .map(|(module_key, module)| {
                let s1 = &module_key.0;
                let s2 = module.type_.to_compact_state().to_string();
                s2 + &s1 + ";"
            })
            .collect()
    }

    fn process_all_pulses(&mut self) -> Vec<(ModuleKey, ModuleKey, Pulse)> {
        let mut result = vec![];

        while !self.pulse_queue.is_empty() {
            let (input_key, output_key, pulse) = self.pulse_queue.pop_front().unwrap();
            result.push((input_key.clone(), output_key.clone(), pulse));

            match pulse {
                High => self.total_high_pulses += 1,
                Low => self.total_low_pulses += 1,
            };
            let module = self.modules.get_mut(&output_key).unwrap();
            let pulse = module.type_.process_pulse(&input_key, &pulse);
            if let Some(pulse) = pulse {
                for key in &module.outputs {
                    self.pulse_queue
                        .push_back((output_key.clone(), key.clone(), pulse));
                }
            }
        }
        result
    }

    fn press_button(&mut self) -> Vec<(ModuleKey, ModuleKey, Pulse)> {
        // Find the broadcaster module.
        let key = ModuleKey("broadcaster".to_string());
        let button_key = ModuleKey("button".to_string());

        assert!(self.pulse_queue.is_empty());
        self.pulse_queue.push_back((button_key, key, Low));
        self.process_all_pulses()
    }

    fn create() -> Self {
        Modules {
            modules: HashMap::new(),
            pulse_queue: VecDeque::new(),
            total_high_pulses: 0,
            total_low_pulses: 0,
        }
    }

    fn add_module(&mut self, line: &str) {
        let (target, outputs) = line.split_once(" -> ").unwrap();

        let (module_key, type_): (ModuleKey, ModuleType) = {
            match target.chars().nth(0) {
                Some('%') => {
                    // This is a flip-flop.
                    let name = target.strip_prefix("%").unwrap().to_string();
                    let type_ = FlipFlop {
                        internal_state: Off,
                    };
                    let name = ModuleKey(name);
                    (name, type_)
                }
                Some('&') => {
                    // This is a NAND.
                    let name = target.strip_prefix("&").unwrap().to_string();

                    // We don't properly initialise the dependencies in NAND gates, this gets done in finalise.
                    let type_ = Nand {
                        input_component_to_last_known_pulse: HashMap::new(),
                        number_of_low_pulses: 0,
                    };
                    let name = ModuleKey(name);
                    (name, type_)
                }
                _ => {
                    if target == "broadcaster" {
                        (ModuleKey(target.to_string()), Broadcaster)
                    } else {
                        panic!("Unrecognised name format - doesn't contain anything.");
                    }
                }
            }
        };

        let outputs: Vec<ModuleKey> = outputs
            .split(", ")
            .map(|s| ModuleKey(s.to_string()))
            .collect();

        let module = Module { type_, outputs };
        self.modules.insert(module_key, module);
    }

    fn finalise(&mut self) {
        // Bit of a hack - we want to fix up some state after we know we've added everything. This includes:
        // - Identifying any sink nodes.
        // - Fixing up NAND gates.

        let module_keys: Vec<ModuleKey> = self.modules.keys().map(|x| x.clone()).collect();
        let mut input_to_output_keys: HashMap<ModuleKey, Vec<ModuleKey>> = HashMap::new();
        for key in module_keys {
            let output_keys = self.modules.get(&key).unwrap().outputs.clone();
            for output_key in output_keys {
                let v = input_to_output_keys.entry(key.clone()).or_insert(vec![]);
                v.push(output_key);
            }
        }

        // Identify and add any sink nodes.
        for output_keys in input_to_output_keys.values() {
            for output_key in output_keys {
                if !self.modules.contains_key(output_key) {
                    let module = Module {
                        type_: Sink,
                        outputs: vec![],
                    };
                    self.modules.insert(output_key.clone(), module);
                }
            }
        }

        // Fix up state of NAND keys.
        for (input_key, output_keys) in input_to_output_keys.into_iter() {
            for output_key in output_keys {
                let output_module = self.modules.get_mut(&output_key).unwrap();
                match &mut output_module.type_ {
                    Nand {
                        input_component_to_last_known_pulse,
                        number_of_low_pulses,
                    } => {
                        input_component_to_last_known_pulse.insert(input_key.clone(), Low);
                        *number_of_low_pulses += 1;
                    }
                    _ => (),
                }
            }
        }
    }

    fn strip_to_dependency_tree_from_node(&mut self, target_key: &ModuleKey) {
        let module_keys: Vec<ModuleKey> = self.modules.keys().map(|x| x.clone()).collect();
        let mut output_to_input_keys: HashMap<ModuleKey, Vec<ModuleKey>> = HashMap::new();

        for key in module_keys {
            let output_keys = self.modules.get(&key).unwrap().outputs.clone();
            for output_key in output_keys {
                let v = output_to_input_keys
                    .entry(output_key.clone())
                    .or_insert(vec![]);
                v.push(key.clone());
            }
        }

        let find_component = |links: &HashMap<ModuleKey, Vec<ModuleKey>>| -> HashSet<ModuleKey> {
            let mut result = HashSet::new();
            let mut process_queue = vec![target_key.clone()];

            while !process_queue.is_empty() {
                let v = process_queue.pop().unwrap();
                let next_ = links.get(&v);
                result.insert(v.clone());

                match next_ {
                    None => (),
                    Some(next_) => {
                        for n in next_ {
                            if !result.contains(n) {
                                process_queue.push(n.clone());
                            }
                        }
                    }
                }
            }

            result
        };

        let in_ = find_component(&output_to_input_keys);

        // Strip out everything that's not in that connected component.
        let old_modules = mem::replace(&mut self.modules, HashMap::new());
        self.modules = old_modules
            .into_iter()
            .filter_map(|(module_key, module)| {
                if in_.contains(&module_key) {
                    Some((module_key, module))
                } else {
                    None
                }
            })
            .collect();
        for module in self.modules.values_mut() {
            module.outputs = module
                .outputs
                .iter()
                .filter(|key| in_.contains(&key))
                .cloned()
                .collect();
        }
    }
}

fn identify_cycle_in_target_node(s: &str, target_key: &str) -> () {
    // Given a target node, identify the strongly connected component, and when that strongly connected component cycles.
    // Print out the length of the cycle, and on which points this node emits a "Low" pulse.
    let mut modules = Modules::create();
    for line in s.lines() {
        modules.add_module(line);
    }
    modules.finalise();

    let target_key = ModuleKey(target_key.to_string());
    modules.strip_to_dependency_tree_from_node(&target_key);

    let mut state = HashMap::<String, usize>::new();
    for i in 1..2_000_000 {
        let pulses = modules.press_button();
        let pulses_from_target_key: Vec<Pulse> = pulses
            .into_iter()
            .filter(|(input_key, _output_key, _pulse)| input_key == &target_key.clone())
            .map(|(_input_key, _output_key, pulse)| pulse)
            .collect();
        if pulses_from_target_key.contains(&Low) {
            println!("Low pulse emitted from this node on button {}", i);
        }

        // Get the compact state
        let compact_state = modules.compact_state();
        if state.contains_key(&compact_state) {
            println!("We have a cycle in the state!");
            println!(
                "Old time we visited this: {}",
                state.get(&compact_state).unwrap()
            );
            println!("This time: {}", i);
            return;
        } else {
            state.insert(compact_state, i);
        }

        if i % 1_000_000 == 0 {
            println!("{}", i);
        }
    }
    panic!("Never found a cycle");
}

fn main() {
    let s = include_str!("input").trim();

    /* Part 1. Create the modules, press the button 1000 times, count how many low/high pulses there have been. */
    let mut modules = Modules::create();
    for line in s.lines() {
        modules.add_module(line);
    }
    modules.finalise();

    for _ in 0..1_000 {
        modules.press_button();
    }
    println!(
        "Solution for part 1: {}",
        modules.total_high_pulses * modules.total_low_pulses
    );

    // Part 2 - the script doesn't fully do everything itself.
    // I plotted the structure in Graphviz, and found that there are 4 strongly connected components that end up feeding into "rx".
    // These nodes all need to emit a Low pulse in a turn for the "rx" node to emit High.
    //
    // This function finds out many turns it takes for one of these strongly connected components to cycle, and also prints out
    // on which turns the node ever emits a Low pulse - we're in the special case where this only happens on the last element
    // of the cycle, like the ghosts problem on day 8. Then I LCM them.
    //
    // This is necessary-but-not-sufficient (we'd also need the Low pulses being delivered in the right order so they hit rx at
    // the same time), but whatever, this works.
    identify_cycle_in_target_node(s, "mr");
    identify_cycle_in_target_node(s, "vv");
    identify_cycle_in_target_node(s, "bl");
    identify_cycle_in_target_node(s, "pv");
}
