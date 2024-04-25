use clap::Parser;
// use argfile::Argument::Path;
use std::{
    collections::HashMap,
    ffi::OsString,
    fs::File,
    io::{self, Read},
    ops::{Index, IndexMut},
    str::{from_utf8, Utf8Error},
};

#[derive(Debug)]
enum Token {
    Plus,
    Minus,
    LeftBracket(usize),
    RightBracket(usize),
    Print,
    Read,
    Left,
    Right,
}

#[derive(Parser, Debug)]
struct Args {
    prog: Option<OsString>,
}

#[derive(Debug)]
enum Brainfuck {
    UnmatchedLeft,
    UnmatchedRight,
}

#[derive(Debug)]
enum Err {
    IO(io::Error),
    Utf(Utf8Error),
    Bf(Brainfuck),
}

fn main() -> Result<(), Err> {
    let args = Args::parse();
    let mut code;
    if let Some(x) = &args.prog {
        println!("{:?}", x);
        let mut file = File::open(x).map_err(Err::IO)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(Err::IO)?;
        println!("{}", from_utf8(&buf).unwrap());

        code = from_utf8(&buf)
            .map_err(Err::Utf)?
            .chars()
            .filter(|x| "+-[].,<>".contains(*x))
            .collect::<Vec<char>>();
    } else {
        code = Vec::new();
    }
    let mut tokens = parse(&code)?;
    println!("{tokens:?}");
    let mut memory = Memory::new();
    let mut pointer = 0;
    let mut index = 0;
    loop {
        while index < tokens.len() {
            match tokens[index] {
                Token::Minus => memory[pointer] = memory[pointer].wrapping_sub(1),
                Token::Plus => memory[pointer] = memory[pointer].wrapping_add(1),
                Token::Left => {
                    pointer -= 1;
                }
                Token::Right => {
                    pointer += 1;
                }
                Token::LeftBracket(x) => {
                    if memory[pointer] == 0 {
                        index = x;
                    }
                }
                Token::RightBracket(x) => {
                    if memory[pointer] != 0 {
                        index = x;
                    }
                }
                Token::Print => {
                    print!("{}", memory[pointer] as char);
                }
                Token::Read => {
                    let mut stdin_handle = io::stdin().lock();
                    let mut byte = [0_u8];
                    stdin_handle.read_exact(&mut byte).unwrap();
                    memory[pointer] = byte[0];
                }
            }
            index += 1;
        }
        if args.prog.is_some() {
            break;
        }
        println!();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).map_err(Err::IO)?;
        if buf.len() == 0 {
          break;
        }
        for c in buf.chars() {
            code.push(c);
        }
        tokens = parse(&code)?;
    }
    Ok(())
}

fn parse(code: &[char]) -> Result<Vec<Token>, Err> {
    let mut tokens = Vec::new();
    'tokenizer: for i in 0..code.len() {
        match code[i] {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '[' => {
                let mut depth = 0;
                for (i, item) in code.iter().enumerate().skip(i) {
                    match item {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ => (),
                    }
                    if depth == 0 {
                        tokens.push(Token::LeftBracket(i));
                        continue 'tokenizer;
                    }
                }
                return Err(Err::Bf(Brainfuck::UnmatchedLeft));
            }
            ']' => {
                let index = tokens.iter().position(|x| match x {
                    Token::LeftBracket(x) => i == *x,
                    _ => false,
                });
                if let Some(x) = index {
                    tokens.push(Token::RightBracket(x));
                } else {
                    return Err(Err::Bf(Brainfuck::UnmatchedRight));
                }
            }
            ',' => tokens.push(Token::Read),
            '.' => tokens.push(Token::Print),
            '<' => tokens.push(Token::Left),
            '>' => tokens.push(Token::Right),
            _ => (),
        }
    }
    Ok(tokens)
}

struct Memory {
    stack: [u8; 1024],
    heap: HashMap<i32, u8>,
}

impl Memory {
    fn new() -> Self {
        Self {
            stack: [0; 1024],
            heap: HashMap::new(),
        }
    }
}

impl Index<i32> for Memory {
    type Output = u8;
    fn index(&self, index: i32) -> &Self::Output {
        if index > 0 && index < 1024 {
            &self.stack[index as usize]
        } else if let Some(x) = self.heap.get(&index) {
            x
        } else {
            &0
        }
    }
}

impl IndexMut<i32> for Memory {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        if index > 0 && index < 1024 {
            &mut self.stack[index as usize]
        } else if self.heap.contains_key(&index) {
            self.heap.get_mut(&index).unwrap()
        } else {
            self.heap.insert(index, 0);
            self.heap.get_mut(&index).unwrap()
        }
    }
}
