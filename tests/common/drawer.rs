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

use citreelo::util::viz_kripke::KripkeStructureGraphvizDrawer;

use crate::common::model::{TestAtomicProp, TestDomainOfAp};






pub struct TestKripkeDrawer {}

impl KripkeStructureGraphvizDrawer<TestDomainOfAp> for TestKripkeDrawer {
    fn get_doap_label(&self,doap : &TestDomainOfAp) -> String {
        let mut strs = Vec::new();
        if doap.atoms.contains(&TestAtomicProp::P) {
            strs.push("P".to_string())
        };
        if doap.atoms.contains(&TestAtomicProp::Q) {
            strs.push("Q".to_string())
        };
        format!(
            "{{{}}}",strs.join(",")
        )
    }
}



