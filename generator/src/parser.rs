//! Generate a parser from the `format.spec` definitions.
//!
//! A state machine is generated to convert the format string
//! in a single pass.

use crate::FormatSpec;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use std::io::{self, Write};

#[derive(Copy, Clone)]
struct Code<'a> {
    code: &'a str,
    header_label: Option<&'a str>,
    header_label_until: Option<u8>,
    sequence: &'a str,
}

enum TreeNode<'a> {
    Code(Code<'a>),

    Partial {
        state: usize,
        nodes: BTreeMap<u8, TreeNode<'a>>,
    },
}

enum Transition<'a> {
    State(usize),
    Code(Code<'a>),
}

struct State<'a> {
    number: usize,
    transitions: Vec<(u8, Transition<'a>)>,
}

/// Generate the parser code.
///
/// If `render_fields` is `true`, the generated parser contains code to read
/// values from history entries. If it is `false`, the parser writes the labels
/// of every resource specifier.
pub fn generate_parser(
    mut output: impl Write,
    specs: &[FormatSpec],
    render_fields: bool,
) -> io::Result<()> {
    let states = state_machine(specs);

    // Compute states.
    let match_branches = states.iter().map(|state| {
        let state_number = state.number;

        let chr_states = state.transitions.iter().map(|(chr, node)| {
            let expr = match node {
                Transition::Code(code) => {
                    let expr = match (render_fields, code.header_label) {
                        (false, Some(label)) => {
                            // Code to write the label of the specifier.

                            let until = match code.header_label_until {
                                Some(byte) => {
                                    // If `[label-until] C` is present, consume
                                    // all bytes until the byte `C` is found.
                                    quote! {
                                        loop {
                                            if matches!(input.next(), None | Some((_, #byte))) {
                                                break;
                                            }
                                        }
                                    }
                                }

                                None => quote! {},
                            };

                            quote! {
                                output.write_all(#label.as_bytes())?;
                                #until
                            }
                        }

                        _ => {
                            // Code to replace a specifier with the value from
                            // a history entry.

                            let tokens: TokenStream = code.code.parse().unwrap();
                            quote! { #tokens }
                        }
                    };

                    quote! {
                        #expr
                        state = 0;
                    }
                }

                Transition::State(state) => {
                    quote! { state = #state }
                }
            };

            quote! {
                #chr => { #expr }
            }
        });

        // When an unknown character is found:
        // - state = 0: write the character to the output.
        // - state ≠ 0: discards the current specifier.
        let unknown_char = if state.number == 0 {
            quote! { output.write_all(&[*chr])?; }
        } else {
            quote! { discard_spec!(); }
        };

        quote! {
            #state_number => {
                match chr {
                    #(#chr_states)*

                    _ => { #unknown_char }
                }
            }
        }
    });

    // `rusage_field!` is only available when `render_fields` is true.
    let rusage_field_macro = if render_fields {
        quote! {
            /// Print a `rusage` field.
            macro_rules! rusage_field {
                ($field:ident) => {{
                    if let State::Finished { rusage, .. } = &entry.state {
                        w!(rusage.$field);
                    }
                }};
            }
        }
    } else {
        quote! {}
    };

    // State to discard the current specifier.
    let discard_spec_state = states.last().unwrap().number + 1;

    // Parser code.
    let code = quote! {{
        let format = format.as_bytes();
        let mut input = format.iter().enumerate();

        let mut state = 0;
        let mut last_index_at_zero = 0;

        /// Write to the output.
        macro_rules! w {
            ($e:expr) => {
                write!(&mut output, "{}", $e)?;
            };

            ($($e:tt)+) => {
                write!(&mut output, $($e)+)?;
            };
        }

        #rusage_field_macro

        while let Some((chr_index, chr)) = input.next() {
            if state == 0 {
                last_index_at_zero = chr_index;
            }

            'current_chr: loop {
                macro_rules! discard_spec {
                    () => {{
                        state = #discard_spec_state;
                        continue 'current_chr;
                    }}
                }

                match state {
                    #(#match_branches)*

                    #discard_spec_state => {
                        if let Some(bytes) = format.get(last_index_at_zero..chr_index) {
                            output.write_all(bytes)?;
                        }

                        state = 0;
                        continue 'current_chr;
                    },

                    _ => { discard_spec!(); },
                }

                break 'current_chr;
            }
        }

        // Copy raw format string if the last specifier was incompleted.
        if state != 0 {
            output.write_all(&format[last_index_at_zero..])?;
        }
    }};

    write!(output, "{}", code)
}

/// Generates a parser from a list of specifiers.
fn state_machine(specs: &[FormatSpec]) -> Vec<State> {
    let mut states_map: BTreeMap<u8, TreeNode> = BTreeMap::new();
    let mut state_counter = 0;

    for spec in specs {
        for seq in &spec.sequences {
            let mut map = &mut states_map;
            let mut chars = seq.bytes().peekable();

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
                        header_label: spec.header_label.as_deref(),
                        header_label_until: spec.header_label_until,
                        sequence: seq,
                    });

                    let old = map.insert(c, item);
                    assert!(old.is_none(), "conflicts: {}", seq);
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
fn flatten_tree<'a>(output: &mut Vec<State<'a>>, state: usize, tree: &BTreeMap<u8, TreeNode<'a>>) {
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
