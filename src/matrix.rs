use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Mul},
    sync::mpsc::{self, Receiver, Sender},
    thread, vec,
};

use crate::{dot_product, Vector};
use anyhow::{anyhow, Result};

const NUM_PROCESS: usize = 4;

#[derive(Debug)]
pub struct Matrix<T: Debug> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

#[derive(Debug)]
pub struct MsgInput<T: Debug> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

impl<T> MsgInput<T>
where
    T: Debug,
{
    fn new(idx: usize, row: impl Into<Vec<T>>, col: impl Into<Vec<T>>) -> Self {
        Self {
            idx,
            row: Vector::new(row.into()),
            col: Vector::new(col.into()),
        }
    }
}

pub struct MsgOutput<T> {
    idx: usize,
    val: T,
}

pub struct Msg<T: Debug> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T>
where
    T: Debug,
{
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    if a.col != b.row {
        return Err(anyhow!("a.col not equal b.row"));
    }

    let senders = (0..NUM_PROCESS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(&msg.input.row, &msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        val: value,
                    }) {
                        eprintln!("{:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = a.row * b.col;
    let mut receivers = Vec::with_capacity(matrix_len);
    for i in 0..a.row {
        for j in 0..b.col {
            let row_data = &a.data[i * a.col..(i + 1) * a.col];
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<T>>();

            let idx = i * b.col + j;
            let input_msg = MsgInput::new(idx, row_data, col_data);
            println!("input_msg:{:?}", input_msg);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input_msg, tx);
            if let Err(e) = senders[idx % NUM_PROCESS].send(msg) {
                eprint!("send {:?}", e);
            }
            receivers.push(rx);
        }
    }

    let mut data = vec![T::default(); matrix_len];
    for rx in receivers {
        let msg = rx.recv()?;
        data[msg.idx] = msg.val;
    }
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

impl<T> Matrix<T>
where
    T: Debug,
{
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("matrix multiply error")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matrix_multiply() {
        let a = Matrix::new(&[1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(&[1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        // assert_eq!(
        //     format!("{:?}", c),
        //     "Matrix { data: [22, 28, 49, 64], row: 2, col: 2 }"
        // )
    }
}
