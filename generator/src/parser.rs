//! Generate a parser from the `format.spec` definitions.
//!
//! A state machine is generated to convert the format string
//! in a single pass.

use crate::FormatSpec;
use std::collections::BTreeMap;
use std::io::{self, Write};

#[derive(Copy, Clone)]
struct Code<'a> {
    code: &'a str,
    sequence: &'a str,
}

enum TreeNode<'a> {
    Code(Code<'a>),

    Partial {
        state: usize,
        nodes: BTreeMap<char, TreeNode<'a>>,
    },
}

enum Transition<'a> {
    State(usize),
    Code(Code<'a>),
}

struct State<'a> {
    number: usize,
    transitions: Vec<(char, Transition<'a>)>,
}

/// Generate the parser code.
pub fn generate_parser(mut output: impl Write, specs: &[FormatSpec]) -> io::Result<()> {
    let states = state_machine(specs);

    // State to discard the current specifier.
    let discard_spec_state = states.last().unwrap().number + 1;
    let discard_spec = format!(
        "{{ state = {}; continue 'current_chr; }}",
        discard_spec_state
    );

    // Header.
    output.write_all(
        b"
            'current_chr: loop {
                match state {
        ",
    )?;

    // Write states.
    for state in states {
        writeln!(output, "{} => {{\nmatch chr {{", state.number)?;

        for (chr, node) in state.transitions {
            writeln!(output, "b'{}' => {{", chr.escape_default())?;

            match node {
                Transition::Code(code) => {
                    writeln!(
                        output,
                        "// '{}'\n{}\nstate = 0;\n}},",
                        code.sequence,
                        code.code.replace("discard_spec!()", &discard_spec)
                    )?;
                }

                Transition::State(state) => {
                    writeln!(output, "state = {};\n}},", state)?;
                }
            }
        }

        let unknown_char = if state.number == 0 {
            "{ output.write_all(&[*chr])?; }"
        } else {
            discard_spec.as_ref()
        };

        writeln!(
            output,
            "
                        _ => {}
                    }} // end of `match chr`
                }}, // state = {}
            ",
            unknown_char, state.number
        )?;
    }

    // Footer.
    writeln!(
        output,
        "
                    {discard_spec_state} => {{
                        if let Some(bytes) = format.get(last_index_at_zero..chr_index) {{
                            output.write_all(bytes)?;
                        }}

                        state = 0;
                        continue 'current_chr;
                    }},

                    _ => {discard_spec},
                }}

                break 'current_chr;
            }} // loop 'current_chr
        ",
        discard_spec_state = discard_spec_state,
        discard_spec = discard_spec
    )?;

    Ok(())
}

/// Generates a parser from a list of specifiers.
fn state_machine(specs: &[FormatSpec]) -> Vec<State> {
    let mut states_map: BTreeMap<char, TreeNode> = BTreeMap::new();
    let mut state_counter = 0;

    for spec in specs {
        for seq in &spec.sequences {
            let mut map = &mut states_map;
            let mut chars = seq.chars().peekable();

            while let Some(c) = chars.next() {
                if chars.peek().is_some() {
                    let entry = map.entry(c).or_insert_with(|| {
                        state_counter += 1;
                        TreeNode::Partial {
                            state: state_counter,
                            nodes: BTreeMap::new(),
                        }
                    });

                    map = match entry {
                        TreeNode::Code(code) => {
                            panic!("conflicts: {} - {}", seq, code.sequence)
                        }

                        TreeNode::Partial { nodes, .. } => nodes,
                    }
                } else {
                    let item = TreeNode::Code(Code {
                        code: &spec.parser_code,
                        sequence: seq,
                    });

                    if map.insert(c, item).is_some() {
                        panic!("conflicts: {}", seq);
                    }
                }
            }
        }
    }

    // Convert the tree to a flatten list.
    let mut states = Vec::new();
    flatten_tree(&mut states, 0, &states_map);
    states.sort_unstable_by_key(|s| s.number);
    states
}

/// Traverse a states tree and produces a flatten list of transitions.
fn flatten_tree<'a>(
    output: &mut Vec<State<'a>>,
    state: usize,
    tree: &BTreeMap<char, TreeNode<'a>>,
) {
    let transitions = tree
        .iter()
        .map(|(chr, node)| {
            let t = match node {
                TreeNode::Code(code) => Transition::Code(*code),
                TreeNode::Partial { state, .. } => Transition::State(*state),
            };

            (*chr, t)
        })
        .collect();

    output.push(State {
        number: state,
        transitions,
    });

    for node in tree.values() {
        if let TreeNode::Partial { state, nodes } = &node {
            flatten_tree(output, *state, nodes);
        }
    }
}
