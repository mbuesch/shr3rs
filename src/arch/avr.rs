// -*- coding: utf-8 -*-
//
// Copyright 2022 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

use core::arch::asm;

#[cfg_attr(feature="__devmode__", inline(never))]
#[cfg_attr(not(feature="__devmode__"), inline(always))]
pub fn shr3(state: u32) -> u32 {
    // Split state to 16 bit pairs. The compiler optimizes this away.
    let mut ab = state as u16;
    let mut cd = (state >> 16) as u16;

    unsafe {
        // Cycles: 2 + 15 + 5 + 20 + 2 = 44
        asm!(
            // Setup
            // Cycles: 2
            "mov    {r0_save}, r0",                     // save r0
            "ldi    {fac}, 32",                         // mul factor

            // y ^= y << 13
            // Cycles: 1 + 1 + 2 + 1 + 2 + (2 * 1) + 2 + (4 * 1) = 15
            "movw   {tab:l}:{tab:h}, {ab:l}:{ab:h}",    // mov ab to temp
            "mov    {tcd:l}, {cd:l}",                   // mov c to temp
            "mul    {tcd:l}, {fac}",                    // r0:r1 = c << 5
            "mov    {tcd:l}, r0",                       // temp_c[7:5] = low 3 bits result
            "mul    {tab:h}, {fac}",                    // r0:r1 = b << 5
            "mov    {tab:h}, r0",                       // temp_b[7:5] = low 3 bits result
            "or     {tcd:l}, r1",                       // temp_c[4:0] = high 5 bits result
            "mul    {tab:l}, {fac}",                    // r0:r1 = a << 5
            "or     {tab:h}, r1",                       // temp_b[4:0] = high 5 bits result
            "eor    {ab:h}, r0",                        // b[7:5] ^= low 3 bits result
            "eor    {cd:l}, {tab:h}",                   // c ^= temp_b << 8
            "eor    {cd:h}, {tcd:l}",                   // d ^= temp_c << 8

            // y ^= y >> 17
            // Cycles: 5
            "movw   {tcd:l}:{tcd:h}, {cd:l}:{cd:h}",    // mov cd to temp
            "lsr    {tcd:h}",                           // temp_d >>= 1
            "ror    {tcd:l}",                           // temp_c >>= 1
            "eor    {ab:l}, {tcd:l}",                   // a ^= temp_c >> 16
            "eor    {ab:h}, {tcd:h}",                   // b ^= temp_d >> 16

            // y ^= y << 5
            // Cycles: (2 * 1) + 2 + 1 + 2 + (2 * 1) + 2 + (2 * 1) + 2 + (5 * 1) = 20
            "movw   {tab:l}:{tab:h}, {ab:l}:{ab:h}",    // mov ab to temp
            "movw   {tcd:l}:{tcd:h}, {cd:l}:{cd:h}",    // mov cd to temp
            "mul    {tcd:h}, {fac}",                    // r0:r1 = d << 5
            "mov    {tcd:h}, r0",                       // temp_d[7:5] = low 3 bits result
            "mul    {tcd:l}, {fac}",                    // r0:r1 = c << 5
            "mov    {tcd:l}, r0",                       // temp_c[7:5] = low 3 bits result
            "or     {tcd:h}, r1",                       // temp_d[4:0] = high 5 bits result
            "mul    {tab:h}, {fac}",                    // r0:r1 = b << 5
            "mov    {tab:h}, r0",                       // temp_b[7:5] = low 3 bits result
            "or     {tcd:l}, r1",                       // temp_c[4:0] = high 5 bits result
            "mul    {tab:l}, {fac}",                    // r0:r1 = a << 5
            "or     {tab:h}, r1",                       // temp_b[4:0] = high 5 bits result
            "eor    {ab:l}, r0",                        // a[7:5] ^= low 3 bits result
            "eor    {ab:h}, {tab:h}",                   // b ^= temp_b
            "eor    {cd:l}, {tcd:l}",                   // c ^= temp_c
            "eor    {cd:h}, {tcd:h}",                   // d ^= temp_d

            // Cleanup
            // Cycles: 2
            "clr    r1",                                // restore r1
            "mov    r0, {r0_save}",                     // restore r0

            ab = inout(reg_pair) ab,                    // input/output byte a + b
            cd = inout(reg_pair) cd,                    // input/output byte c + d
            tab = out(reg_pair) _,                      // temporary byte a + b
            tcd = out(reg_pair) _,                      // temporary byte c + d
            fac = out(reg_upper) _,                     // mul factor
            r0_save = out(reg) _,                       // r0 restore

            options(pure, nomem, nostack),              // We only access registers
        );
    }

    // Combine the 16 bit state pairs. The compiler optimizes this away.
    (ab as u32) | ((cd as u32) << 16)
}

// vim: ts=4 sw=4 expandtab
