use std::{
    f32::consts::PI,
    sync::mpsc,
    thread::{self, Thread},
};

use anyhow::{anyhow, Result};
use rand;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

const NUM_PROCESS: usize = 4;
fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();
    for i in 0..NUM_PROCESS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }

    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
    });

    consumer
        .join()
        .map_err(|e| anyhow!("Thread Join err:{:?}", e))?;

    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;
        thread::sleep(std::time::Duration::from_millis(1000))
    }
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}
