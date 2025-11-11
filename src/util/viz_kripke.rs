/*
Copyright 2025 Erwan Mahe (github.com/erwanM974)

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/


use graphviz_dot_builder::{edge::edge::GraphVizEdge, graph::graph::GraphVizDiGraph, item::node::{node::GraphVizNode, style::{GraphvizNodeStyleItem, GvNodeShape}}, traits::DotBuildable};

use crate::kripke::KripkeStructure;



pub trait KripkeStructureGraphvizDrawer<DOAP> {

    fn get_doap_label(&self,doap : &DOAP) -> String;

    fn get_kripke_repr(&self,kripke : &KripkeStructure<DOAP>) -> GraphVizDiGraph {

        let mut digraph = GraphVizDiGraph::new(vec![]);
        
        for (st_id,state) in kripke.states.iter().enumerate() {
            let label = self.get_doap_label(&state.value_in_domain);
            let style = vec![
                GraphvizNodeStyleItem::Shape(GvNodeShape::Circle),
                GraphvizNodeStyleItem::Label(format!("s{}\n{}",st_id,label))];
            digraph.add_node(GraphVizNode::new(format!("st{:}",st_id),style));
        }
        for (orig_st_id,orig_st) in kripke.states.iter().enumerate() {
            for targ_st_id in orig_st.outgoing_transitions_targets.iter() {
                let edge = GraphVizEdge::new(
                    format!("st{:}",orig_st_id),
                    None,
                    format!("st{:}",targ_st_id),
                    None,
                    Vec::new()
                );
                digraph.add_edge(edge);
            }
        }
        digraph
        
    }
}