#![feature(destructuring_assignment)]

// pub mod bus;
#[allow(dead_code)]
pub mod cpu;
pub mod memory;


#[derive(Debug)]
pub enum Either<L,R> {
    Left(L),
    Right(R),
}

