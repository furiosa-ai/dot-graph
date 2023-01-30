use crate::graphviz::{
    agfstnode, agfstout, agfstsubg, agget, agnameof, agnxtattr, agnxtnode, agnxtout, agnxtsubg,
    agread, fopen, Agedge_s, Agnode_s, Agraph_s, Agsym_s,
};
use crate::{
    edge::edge::Edge,
    graph::{
        igraph::IGraph,
        graph::Graph,
    },
    node::node::Node,
};
use std::boxed::Box;
use std::collections::{BTreeMap, HashSet};
use std::ffi::{CStr, CString};

unsafe fn c_to_rust_string(ptr: *const i8) -> String {
    String::from_utf8_lossy(CStr::from_ptr(ptr).to_bytes()).to_string()
}

pub fn parse(path: &str) -> Graph {
    let path = CString::new(path).unwrap();
    let option = CString::new("r").unwrap();
    unsafe {
        let fp = fopen(path.as_ptr(), option.as_ptr());

        let graph = agread(fp as _, 0 as _);
        parse_graph(graph)
    }
}

pub fn parse_graph(graph: *mut Agraph_s) -> Graph {
    let id = parse_name(graph as _);

    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let root = parse_igraph(graph, &mut nodes, &mut edges);

    Graph::new(id, root, Vec::from_iter(nodes), Vec::from_iter(edges))
}

pub fn parse_igraph(
    graph: *mut Agraph_s,
    nodes_visited: &mut HashSet<Node>,
    edges_visited: &mut HashSet<Edge>,
) -> IGraph {
    let id = parse_name(graph as _);

    // parse subgraphs
    let mut subgraphs = Vec::new();
    unsafe {
        let mut subgraph = agfstsubg(graph);
        while !subgraph.is_null() {
            let graph = parse_igraph(subgraph, nodes_visited, edges_visited);

            subgraphs.push(Box::new(graph));
            subgraph = agnxtsubg(subgraph);
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

    // parse nodes and edges
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    unsafe {
        let mut node = agfstnode(graph);
        while !node.is_null() {
            let (n, es) = parse_node(node, graph, &nkeys, &ekeys);
            if !nodes_visited.contains(&n) {
                nodes_visited.insert(n.clone());
                nodes.push(n);
            }
            for e in es {
                if !edges_visited.contains(&e) {
                    edges_visited.insert(e.clone());
                    edges.push(e);
                }
            }

            node = agnxtnode(graph, node);
        }
    };

    IGraph {
        id,
        subgraphs,
        nodes,
        edges,
    }
}

pub fn parse_node(
    node: *mut Agnode_s,
    graph: *mut Agraph_s,
    nkeys: &[*mut i8],
    ekeys: &[*mut i8],
) -> (Node, Vec<Edge>) {
    let id = parse_name(node as _);

    let mut attrs = BTreeMap::new();
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
            let e = parse_edge(edge, node, &ekeys);
            edges.push(e);

            edge = agnxtout(graph, edge);
        }
    };

    let node = Node { id, attrs };

    (node, edges)
}

pub fn parse_edge(edge: *mut Agedge_s, node: *mut Agnode_s, ekeys: &[*mut i8]) -> Edge {
    let from = parse_name(node as _);
    let to = unsafe { parse_name((*edge).node as _) };

    let mut attrs = BTreeMap::new();
    for &key in ekeys {
        let (key, value) = unsafe {
            let value = agget(edge as _, key);
            (c_to_rust_string(key), c_to_rust_string(value))
        };
        if !value.is_empty() {
            attrs.insert(key, value);
        }
    }

    Edge { from, to, attrs }
}

pub fn parse_name(obj: *mut ::std::os::raw::c_void) -> String {
    unsafe { c_to_rust_string(agnameof(obj)) }
}
