// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use std::io::{stdout, Write};

pub struct ProgressBar {
    size: usize,
    progress: usize,
}

impl ProgressBar {
    pub fn new(size: usize) -> Self {
        print!(
            "[__________________________________________________] ({} bytes)\r[",
            size
        );
        ProgressBar { size, progress: 0 }
    }

    pub fn progress(&mut self, progress: usize) {
        let current = self.progress * 50 / self.size;
        self.progress = progress;
        let updated = self.progress * 50 / self.size;
        for _ in current..updated {
            print!("#");
        }
        stdout().flush().unwrap();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        println!();
    }
}
