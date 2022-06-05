// -*- coding: utf-8 -*-
//
// Copyright 2022 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

#[cfg_attr(feature="__devmode__", inline(never))]
#[cfg_attr(not(feature="__devmode__"), inline(always))]
#[cfg_attr(target_arch="avr", allow(dead_code))]
pub fn shr3(mut state: u32) -> u32 {
    // Fixed variant with full cycle.
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state
}

// vim: ts=4 sw=4 expandtab
