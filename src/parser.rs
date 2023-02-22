use crate::graphviz::{
    agfstnode, agfstout, agfstsubg, agget, agisdirected, agnameof, agnxtattr, agnxtnode, agnxtout,
    agnxtsubg, agread, fopen, Agedge_s, Agnode_s, Agraph_s, Agsym_s,
};
use crate::{
    edge::Edge,
    error::DotGraphError,
    graphs::{Graph, IGraph},
    node::Node,
};

use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::path::Path;

unsafe fn c_to_rust_string(ptr: *const i8) -> String {
    String::from_utf8_lossy(CStr::from_ptr(ptr).to_bytes()).to_string()
}

/// Parse the given dot format file in `path`.
///
/// # Arguments
///
/// * `path` - Path to the dot file in `&str`
///
/// # Returns
///
/// `Err` if the given file is not a graph or is not a DAG,
/// otherwise `Ok` with the parsed graph.
pub fn parse(path: &str) -> Result<Graph, DotGraphError> {
    if !Path::new(path).exists() {
        return Err(DotGraphError::InvalidGraph(String::from(path)));
    }

    let cpath = CString::new(path).unwrap();
    let coption = CString::new("r").unwrap();
    unsafe {
        let fp = fopen(cpath.as_ptr(), coption.as_ptr());

        let graph = agread(fp as _, 0 as _);
        if graph.is_null() {
            return Err(DotGraphError::InvalidGraph(String::from(path)));
        }
        if agisdirected(graph) == 0 {
            return Err(DotGraphError::UndirectedGraph(String::from(path)));
        }

        parse_graph(graph)
    }
}

fn parse_graph(graph: *mut Agraph_s) -> Result<Graph, DotGraphError> {
    let id = parse_name(graph as _);

    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let root = parse_igraph(graph, &mut nodes, &mut edges);

    Graph::new(id, root, nodes, edges)
}

fn parse_igraph(
    graph: *mut Agraph_s,
    nodes_visited: &mut HashSet<Node>,
    edges_visited: &mut HashSet<Edge>,
) -> IGraph {
    let id = parse_name(graph as _);

    // parse subgraphs
    let mut igraphs = HashSet::new();
    unsafe {
        let mut subgraph = agfstsubg(graph);
        while !subgraph.is_null() {
            igraphs.insert(parse_igraph(subgraph, nodes_visited, edges_visited));
            subgraph = agnxtsubg(subgraph);
        }
    };

    // parse graph attr names
    let mut gkeys = Vec::new();
    unsafe {
        let mut key = agnxtattr(graph, 0, std::ptr::null_mut::<Agsym_s>());
        while !key.is_null() {
            gkeys.push((*key).name);
            key = agnxtattr(graph, 0, key);
        }
    };

    // parse node attr names
    let mut nkeys = Vec::new();
    unsafe {
        let mut key = agnxtattr(graph, 1, std::ptr::null_mut::<Agsym_s>());
        while !key.is_null() {
            nkeys.push((*key).name);
            key = agnxtattr(graph, 1, key);
        }
    };

    // parse edge attr names
    let mut ekeys = Vec::new();
    unsafe {
        let mut key = agnxtattr(graph, 2, std::ptr::null_mut::<Agsym_s>());
        while !key.is_null() {
            ekeys.push((*key).name);
            key = agnxtattr(graph, 2, key);
        }
    };

    // parse graph attrs
    let mut attrs = HashMap::new();
    for key in gkeys {
        let (key, value) = unsafe {
            let value = agget(graph as _, key);
            (c_to_rust_string(key), c_to_rust_string(value))
        };
        if !value.is_empty() {
            attrs.insert(key, value);
        }
    }

    // parse nodes and edges
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    unsafe {
        let mut node = agfstnode(graph);
        while !node.is_null() {
            let (n, es) = parse_node(node, graph, &nkeys, &ekeys);
            if !nodes_visited.contains(&n) {
                nodes_visited.insert(n.clone());
                nodes.insert(n);
            }
            for e in es {
                if !edges_visited.contains(&e) {
                    edges_visited.insert(e.clone());
                    edges.insert(e);
                }
            }

            node = agnxtnode(graph, node);
        }
    };

    IGraph::new(id, igraphs, nodes, edges, attrs)
}

fn parse_node(
    node: *mut Agnode_s,
    graph: *mut Agraph_s,
    nkeys: &[*mut i8],
    ekeys: &[*mut i8],
) -> (Node, Vec<Edge>) {
    let id = parse_name(node as _);

    let mut attrs = HashMap::new();
    for &key in nkeys {
        let (key, value) = unsafe {
            let value = agget(node as _, key);
            (c_to_rust_string(key), c_to_rust_string(value))
        };
        if !value.is_empty() {
            attrs.insert(key, value);
        }
    }

    let mut edges = Vec::new();
    unsafe {
        let mut edge = agfstout(graph, node);
        while !edge.is_null() {
            let e = parse_edge(edge, node, ekeys);
            edges.push(e);

            edge = agnxtout(graph, edge);
        }
    };

    let node = Node::new(id, attrs);

    (node, edges)
}

fn parse_edge(edge: *mut Agedge_s, node: *mut Agnode_s, ekeys: &[*mut i8]) -> Edge {
    let from = parse_name(node as _);
    let to = unsafe { parse_name((*edge).node as _) };
    let id = (from, to);

    let mut attrs = HashMap::new();
    for &key in ekeys {
        let (key, value) = unsafe {
            let value = agget(edge as _, key);
            (c_to_rust_string(key), c_to_rust_string(value))
        };
        if !value.is_empty() {
            attrs.insert(key, value);
        }
    }

    Edge::new(id, attrs)
}

fn parse_name(obj: *mut ::std::os::raw::c_void) -> String {
    unsafe { c_to_rust_string(agnameof(obj)) }
}
