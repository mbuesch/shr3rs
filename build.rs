// -*- coding: utf-8 -*-
//
// Copyright 2022 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

fn main() {
    let ac = autocfg::new();
    ac.emit_has_type("u128");
    autocfg::rerun_path("build.rs");
}

// vim: ts=4 sw=4 expandtab
