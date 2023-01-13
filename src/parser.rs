use std::collections::HashMap;
use crate::ast::graph::Graph;
use crate::ast::graph::SubGraph;
use crate::ast::node::Node;
use graphviz_ffi::{ 
    Agraph_s, Agnode_s, Agedge_s, Agsym_s,
    fopen, agread, agget, 
    agfstsubg, agnxtsubg, agparent,
    agfstnode, agnxtnode, 
    agfstedge, agnxtedge,
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
    let subgraphs = parse_subgraph(graph);

    let keys = unsafe {
        // fetch node attr names
        let mut keys = Vec::new();
        let mut key = agnxtattr(graph, 1, 0 as *mut Agsym_s);
        while !key.is_null() {
            keys.push((*key).name);
            key = agnxtattr(graph, 1, key);
        }

        keys
    };

    let nodes = unsafe {
        // parse nodes
        let mut nodes = Vec::new();
        let mut node = agfstnode(graph);
        while !node.is_null() {
            let n = parse_node(node, &keys);
            nodes.push(n);

            node = agnxtnode(graph, node);
        }

        nodes
    };

    Graph { subgraphs, nodes, edges: Vec::new() }
}

pub fn parse_subgraph(graph: *mut Agraph_s) -> Vec<SubGraph> {
    let id = parse_name(graph as _);
    let parent = unsafe {
        let parent = agparent(graph);
        if parent.is_null() {
            "root".to_string()
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

pub fn parse_node(node: *mut Agnode_s, keys: &Vec<*mut i8>) -> Node {
    let id = parse_name(node as _);

    let attrs = unsafe {
        let mut attrs = HashMap::new();

        for key in keys {
            let value = agget(node as _, *key);

            let key = to_rust_string!(*key);
            let value = to_rust_string!(value);
            attrs.insert(key, value);
        }

        attrs
    };

    Node { id, attrs }
}

pub fn parse_edge(edge: *mut Agedge_s) {
    let name = parse_name(edge as _);
    println!("{}", name);
}

pub fn parse_name(obj: *mut ::std::os::raw::c_void) -> String {
    unsafe {
        let name = agnameof(obj);
        to_rust_string!(name)
    }
}
