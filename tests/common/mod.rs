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

// Each integration-test binary compiles this module independently and
// none of them uses every helper, so dead-code warnings are noise here.
#![allow(dead_code)]

pub mod asserts;
pub mod drawer;
pub mod generators;
pub mod model;
pub mod oracle;
pub mod parser;
pub mod zoo;
