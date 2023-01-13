use std::ffi::CStr;
use graphviz_ffi::{ 
    Agraph_s,
    fopen, agread, agfstnode, agnameof };

macro_rules! to_c_string {
    ($str:expr) => {
        std::ffi::CString::new($str).unwrap().as_ptr()
    };
}

pub fn parse(path: &str) {
    unsafe {
        let fp = fopen(to_c_string!(path), to_c_string!("r"));

        let graph = agread(fp as _, 0 as _);
        parse_graph(graph);
    }
}

pub fn parse_graph(graph: *mut Agraph_s) {
    unsafe {
        let node = agfstnode(graph);
        let name = {
            let name = agnameof(node as _);
            let cstr = CStr::from_ptr(name);
            String::from_utf8_lossy(cstr.to_bytes()).to_string()
        };
        println!("{}", name);
    }
}
