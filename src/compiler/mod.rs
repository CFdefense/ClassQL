/*
    This module contains the compiler for the application.
    The compiler is responsible for compiling the user's query into sql.
*/

pub mod compiler;
pub mod lexer;
pub mod token;
pub mod parser;