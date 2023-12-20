use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone)]
struct PartRange {
    x: Range<usize>,
    m: Range<usize>,
    a: Range<usize>,
    s: Range<usize>,
}

#[derive(Debug)]
enum Which {
    X,
    M,
    A,
    S,
}
use Which::*;

#[derive(Debug)]
enum LesserOrGreater {
    Lesser,
    Greater,
}
use LesserOrGreater::*;

#[derive(Debug)]
struct Condition {
    which_direction_is_true: LesserOrGreater,
    which: Which,
    value: usize,
}

// Split into a true and false part.
fn split(condition: &Condition, part: &PartRange) -> (Option<PartRange>, Option<PartRange>) {
    let PartRange { x, m, a, s } = part;
    let Condition {
        which_direction_is_true,
        which,
        value,
    } = condition;

    let replace = |which: &Which, range: Range<usize>| -> PartRange {
        let x = x.clone();
        let m = m.clone();
        let a = a.clone();
        let s = s.clone();
        match which {
            X => PartRange { x: range, m, a, s },
            M => PartRange { m: range, x, a, s },
            A => PartRange { a: range, x, m, s },
            S => PartRange { s: range, x, m, a },
        }
    };

    let input_range = match which {
        X => x,
        M => m,
        A => a,
        S => s,
    };

    // Split the range at the value.
    let true_and_false_ranges = {
        // I feel like I could have tried to combine these in some way, but they're not exact opposites of
        // each other (because of the possibility of being equal to the value). Oh well, this works.
        match which_direction_is_true {
            Lesser => {
                if input_range.end <= *value {
                    // Everything is below the value, so we're all true.
                    (Some(input_range.clone()), None)
                } else if input_range.start >= *value {
                    // Everything is above or equal to the value, so we're all false.
                    (None, Some(input_range.clone()))
                } else {
                    // The value's in the middle. Both the checks above imply these
                    // ranges are non-empty.
                    let lower = input_range.start..*value;
                    let upper = *value..input_range.end;
                    (Some(lower), Some(upper))
                }
            }
            Greater => {
                if input_range.start > *value {
                    // Everything is above the value, so we're all true.
                    (Some(input_range.clone()), None)
                } else if input_range.end <= *value + 1 {
                    // Everything is below the value, so we're all false. We have plus 1 here
                    // because being equal to the value still counts as not being greater.
                    (None, Some(input_range.clone()))
                } else {
                    // The value's in the middle.
                    let lower = input_range.start..(*value + 1);
                    let upper = *value + 1..input_range.end;
                    (Some(upper), Some(lower))
                }
            }
        }
    };

    let map_range = |r: Option<Range<usize>>| r.map(|r| replace(which, r));

    (
        map_range(true_and_false_ranges.0),
        map_range(true_and_false_ranges.1),
    )
}

#[derive(Copy, Clone, Debug)]
enum FinalResult {
    Accepted,
    Rejected,
}
use FinalResult::*;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct WorkflowKey(String);
#[derive(Debug, Clone)]
enum WorkflowResult {
    AnotherWorkflow(WorkflowKey),
    Finished(FinalResult),
}
use WorkflowResult::*;
#[derive(Debug)]
struct Workflow {
    conditional_workflows: Vec<(Condition, WorkflowResult)>,
    if_none_match: WorkflowResult,
}

#[derive(Debug)]
struct Workflows(HashMap<WorkflowKey, Workflow>);

fn run_one_workflow(workflow: &Workflow, part: PartRange) -> Vec<(PartRange, WorkflowResult)> {
    let mut results = vec![];
    let mut in_progress = vec![part];

    for (condition, result) in &workflow.conditional_workflows {
        let mut new_in_progress = vec![];

        for part in in_progress {
            let (true_section, false_section) = split(condition, &part);

            match true_section {
                Some(x) => results.push((x, result.clone())),
                None => (),
            };

            match false_section {
                Some(x) => new_in_progress.push(x),
                None => (),
            };
        }

        in_progress = new_in_progress;
    }

    // Anything that hasn't been matched so far goes to the final result.
    for part in in_progress {
        results.push((part, workflow.if_none_match.clone()));
    }

    results
}

fn run_workflows(workflows: &Workflows, part: &PartRange) -> Vec<(PartRange, FinalResult)> {
    let start_key = WorkflowKey("in".to_string());

    let mut finished: Vec<(PartRange, FinalResult)> = vec![];
    let mut stack: Vec<(PartRange, WorkflowKey)> = vec![(part.clone(), start_key)];

    while !stack.is_empty() {
        let (part, workflow_key) = stack.pop().unwrap();
        let workflow = workflows.0.get(&workflow_key).expect("Workflow not found");
        let results = run_one_workflow(workflow, part);

        for (part, result) in results {
            match result {
                AnotherWorkflow(key) => stack.push((part, key)),
                Finished(result) => finished.push((part, result)),
            }
        }
    }
    finished
}

fn parse_result(s: &str) -> WorkflowResult {
    match s {
        "R" => Finished(Rejected),
        "A" => Finished(Accepted),
        _ => AnotherWorkflow(WorkflowKey(s.to_string())),
    }
}

fn parse(s: &str) -> (Workflows, Vec<PartRange>) {
    let [workflows_string, parts_string]: [&str; 2] = s
        .trim()
        .split("\n\n")
        .collect::<Vec<&str>>()
        .try_into()
        .expect("More than one double line break found");

    let mut workflows = Workflows(HashMap::new());

    // Each workflow is of the form NAME{CONDITIONED, CONDITIONED, ..., CONDITIONED, FINAL}
    // where CONDITIONED is of the form CONDITION:RESULT, and the final one is just a result.
    for line in workflows_string.lines() {
        let line = line.strip_suffix("}").unwrap();
        let (name, rest) = line.split_once("{").unwrap();
        let name = WorkflowKey(name.to_string());

        let mut conditional_workflows = vec![];

        let parts: Vec<&str> = rest.split(",").collect();
        // Do the condition parts.
        for i in 0..(parts.len() - 1) {
            let conditional_branch = parts[i];
            let (condition, result) = conditional_branch.split_once(":").unwrap();
            let result = parse_result(result);
            let value: usize = condition[2..].parse().unwrap();

            let which = match condition.chars().nth(0).unwrap() {
                'x' => X,
                'm' => M,
                'a' => A,
                's' => S,
                _ => panic!("Couldn't parse as x/m/a/s"),
            };
            let lesser_or_greater = {
                match condition.chars().nth(1).unwrap() {
                    '>' => Greater,
                    '<' => Lesser,
                    _ => panic!("Could not parse as > or <"),
                }
            };
            let condition = Condition {
                which_direction_is_true: lesser_or_greater,
                which,
                value,
            };

            conditional_workflows.push((condition, result));
        }

        // And parse the final result
        let if_none_match = parse_result(parts[parts.len() - 1]);

        let workflow = Workflow {
            conditional_workflows,
            if_none_match,
        };

        workflows.0.insert(name, workflow);
    }

    let mut parts = vec![];

    // TODO: This.
    for part in parts_string.lines() {
        let part_pieces = part
            .strip_prefix("{")
            .unwrap()
            .strip_suffix("}")
            .unwrap()
            .split(",");

        let mut part = PartRange {
            x: 0..0,
            m: 0..0,
            a: 0..0,
            s: 0..0,
        };

        for part_piece in part_pieces {
            let (which, value) = part_piece.split_once("=").unwrap();
            let value: usize = value.parse().unwrap();
            let value = value..(value + 1);
            match which {
                "x" => part.x = value,
                "m" => part.m = value,
                "a" => part.a = value,
                "s" => part.s = value,
                _ => panic!("Couldn't parse as key"),
            }
        }

        parts.push(part);
    }

    (workflows, parts)
}

fn main() {
    let s = include_str!("input");
    let (workflows, parts) = parse(s);

    // Part 1
    let total: usize = parts
        .iter()
        .filter(|part| {
            let results = run_workflows(&workflows, part);

            // Because we passed in single ranges, the result here should be a vector of length 1.
            assert!(results.len() == 1);
            match results[0].1 {
                Accepted => true,
                Rejected => false,
            }
        })
        .map(|part| part.x.start + part.m.start + part.a.start + part.s.start)
        .sum();

    println!("Result for part 1: {}", total);

    // Part 2
    let big_range = PartRange {
        x: 1..4001,
        m: 1..4001,
        a: 1..4001,
        s: 1..4001,
    };
    let results = run_workflows(&workflows, &big_range);
    let total: usize = results
        .iter()
        .map(|(part, accepted)| match accepted {
            Rejected => 0,
            Accepted => part.x.len() * part.m.len() * part.a.len() * part.s.len(),
        })
        .sum();
    println!("Result for part 2: {}", total);
}
