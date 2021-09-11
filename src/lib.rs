#![feature(destructuring_assignment)]
#![feature(never_type)]

// pub mod bus;
#[allow(dead_code)]
pub mod cpu;
pub mod memory;
pub mod utils;

#[derive(Debug)]
pub enum Either<L,R> {
    Left(L),
    Right(R),
}

