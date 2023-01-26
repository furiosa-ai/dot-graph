use std::collections::{ HashSet, BTreeMap };
use std::boxed::Box;
use crate::structs::{ Graph, IGraph, Node, Edge };
use graphviz_ffi::{ 
    Agraph_s, Agnode_s, Agedge_s, Agsym_s,
    fopen, agread, agget, 
    agfstsubg, agnxtsubg,
    agfstnode, agnxtnode, 
    agfstout, agnxtout,
    agnxtattr,
    agnameof };

macro_rules! to_c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

macro_rules! to_rust_string {
    ($bool:expr) => {
        String::from_utf8_lossy(std::ffi::CStr::from_ptr($bool).to_bytes()).to_string()
    };
}

pub fn parse(path: &str) -> Graph {
    let graph = unsafe {
        let fp = fopen(to_c_string!(path), to_c_string!("r"));

        let graph = agread(fp as _, 0 as _);
        parse_graph(graph)
    };

    graph
}

pub fn parse_graph(graph: *mut Agraph_s) -> Graph {
    let id = parse_name(graph as _);

    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let root = parse_igraph(graph, &mut nodes, &mut edges);

    Graph::new(id, root, Vec::from_iter(nodes), Vec::from_iter(edges))
}

pub fn parse_igraph(graph: *mut Agraph_s, nodes_visited: &mut HashSet<Node>, edges_visited: &mut HashSet<Edge>) -> IGraph {
    let id = parse_name(graph as _);

    // parse subgraphs
    let subgraphs = unsafe {
        let mut subgraphs = Vec::new();
        let mut subgraph = agfstsubg(graph);
        while !subgraph.is_null() {
            let graph = parse_igraph(subgraph, nodes_visited, edges_visited);

            subgraphs.push(Box::new(graph));
            subgraph = agnxtsubg(subgraph);
        }

        subgraphs
    };

    // parse node attr names
    let nkeys = unsafe {
        let mut keys = Vec::new();
        let mut key = agnxtattr(graph, 1, 0 as *mut Agsym_s);
        while !key.is_null() {
            keys.push((*key).name);
            key = agnxtattr(graph, 1, key);
        }

        keys
    };

    let ekeys = unsafe {
        let mut keys = Vec::new();
        let mut key = agnxtattr(graph, 2, 0 as *mut Agsym_s);
        while !key.is_null() {
            keys.push((*key).name);
            key = agnxtattr(graph, 2, key);
        }

        keys
    };

    // parse nodes and edges
    let (nodes, edges) = unsafe { 
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
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

        (nodes, edges)
    };

    IGraph { id, subgraphs, nodes, edges }
}

pub fn parse_node(node: *mut Agnode_s, graph: *mut Agraph_s, nkeys: &Vec<*mut i8>, ekeys: &Vec<*mut i8>) -> (Node, Vec<Edge>) {
    let id = parse_name(node as _);

    let attrs = unsafe {
        let mut attrs = BTreeMap::new();

        for key in nkeys {
            let value = agget(node as _, *key);

            let key = to_rust_string!(*key);
            let value = to_rust_string!(value);
            if !value.is_empty() {
                attrs.insert(key, value);
            }
        }

        attrs
    };

    let edges = unsafe {
        let mut edges = Vec::new();
        let mut edge = agfstout(graph, node);
        while !edge.is_null() {
            let e = parse_edge(edge, node, ekeys);
            edges.push(e);

            edge = agnxtout(graph, edge);
        }

        edges
    };

    let node = Node { id, attrs };

    (node, edges)
}

pub fn parse_edge(edge: *mut Agedge_s, node: *mut Agnode_s, ekeys: &Vec<*mut i8>) -> Edge {
    let from = parse_name(node as _);
    let to = unsafe {
        parse_name((*edge).node as _)
    };

    let attrs = unsafe {
        let mut attrs = BTreeMap::new();

        for key in ekeys {
            let value = agget(edge as _, *key);

            let key = to_rust_string!(*key);
            let value = to_rust_string!(value);
            if !value.is_empty() {
                attrs.insert(key, value);
            }
        }

        attrs
    };

    Edge { from, to, attrs }
}

pub fn parse_name(obj: *mut ::std::os::raw::c_void) -> String {
    unsafe {
        let name = agnameof(obj);
        to_rust_string!(name)
    }
}
