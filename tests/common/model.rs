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



use std::collections::HashSet;

use citreelo::kripke::AtomicProposition;








#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TestAtomicProp {
    P, 
    Q
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TestDomainOfAp {
    pub atoms : HashSet<TestAtomicProp>
}

impl TestDomainOfAp {
    pub fn new(atoms: HashSet<TestAtomicProp>) -> Self {
        Self { atoms }
    }
}

impl AtomicProposition<TestDomainOfAp> for TestAtomicProp {
    fn is_satisfied_on_state_domain(&self, state_domain : &TestDomainOfAp) -> bool {
        state_domain.atoms.contains(self)
    }
}