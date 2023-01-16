use std::collections::{ HashSet, BTreeMap };
use crate::structs::{ Graph, SubGraph, Node, Edge };
use graphviz_ffi::{ 
    Agraph_s, Agnode_s, Agedge_s, Agsym_s,
    fopen, agread, agget, 
    agfstsubg, agnxtsubg, agparent,
    agfstnode, agnxtnode, 
    agfstout, agnxtout,
    agnxtattr,
    agroot, agnameof };

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
    // parse subgraphs
    let subgraphs = parse_subgraph(graph);

    // parse node attr names
    let keys = unsafe {
        let mut keys = Vec::new();
        let mut key = agnxtattr(graph, 1, 0 as *mut Agsym_s);
        while !key.is_null() {
            keys.push((*key).name);
            key = agnxtattr(graph, 1, key);
        }

        keys
    };

    // parse nodes and edges
    let (nodes, edges) = unsafe { 
        let subgraphs: Vec<String> = {
            let subgraphs: HashSet<String> = subgraphs.iter().map(|s| s.id.clone()).collect();
            let mut subgraphs = Vec::from_iter(subgraphs);
            subgraphs.sort_by(|a, b| b.len().cmp(&a.len()));

            subgraphs
        }; 
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node = agfstnode(graph);
        while !node.is_null() {
            let (n, mut es) = parse_node(node, graph, &subgraphs, &keys);
            nodes.push(n);
            edges.append(&mut es);

            node = agnxtnode(graph, node);
        }

        (nodes, edges)
    };

    Graph::new(subgraphs, nodes, edges)
}

pub fn parse_subgraph(graph: *mut Agraph_s) -> Vec<SubGraph> {
    let id = parse_name(graph as _);
    let parent = unsafe {
        let parent = agparent(graph);
        if parent.is_null() {
            id.clone()
        } else {
            parse_name(parent as _)
        }
    };
    let subgraph = SubGraph { id, parent };

    let mut subgraphs = unsafe {
        let mut subgraphs = Vec::new();
        let mut subgraph = agfstsubg(graph);
        while !subgraph.is_null() {
            subgraphs.append(&mut parse_subgraph(subgraph));
            subgraph = agnxtsubg(subgraph);
        }

        subgraphs
    };

    subgraphs.push(subgraph);

    subgraphs
}

pub fn parse_node(node: *mut Agnode_s, graph: *mut Agraph_s, subgraphs: &Vec<String>, keys: &Vec<*mut i8>) -> (Node, Vec<Edge>) {
    let id = parse_name(node as _);
    
    // TODO
    // this performs longest prefix match for finding parent subgraph of a node
    // it may not work with general IR graphs
    let parent = {
        let mut parent = unsafe {
            let parent = agroot(node as _);
            parse_name(parent as _)
        };
        for subgraph in subgraphs {
            let prefix = if subgraph.starts_with("cluster_") {
                &subgraph[8..]
            } else {
                subgraph.as_str()
            };
            if id.starts_with(prefix) {
                parent = subgraph.clone();
                break;
            }
        }

        parent
    };

    let attrs = unsafe {
        let mut attrs = BTreeMap::new();

        for key in keys {
            let value = agget(node as _, *key);

            let key = to_rust_string!(*key);
            let value = to_rust_string!(value);
            attrs.insert(key, value);
        }

        attrs
    };

    let edges = unsafe {
        let mut edges = Vec::new();
        let mut edge = agfstout(graph, node);
        while !edge.is_null() {
            let e = parse_edge(edge, node);
            edges.push(e);

            edge = agnxtout(graph, edge);
        }

        edges
    };

    let node = Node { id, parent, attrs };

    (node, edges)
}

pub fn parse_edge(edge: *mut Agedge_s, node: *mut Agnode_s) -> Edge {
    let from = parse_name(node as _);
    let to = unsafe {
        parse_name((*edge).node as _)
    };

    Edge { from, to }
}

pub fn parse_name(obj: *mut ::std::os::raw::c_void) -> String {
    unsafe {
        let name = agnameof(obj);
        to_rust_string!(name)
    }
}
