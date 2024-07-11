use std::error::Error;
use std::fmt::Debug;
use std::io::{stdin, BufRead, BufReader, Read};

fn main() {
    if let Err(e) = run(Default::default(), stdin().lock()) {
        eprintln!("error: {e}");
    }
}

#[derive(Default)]
struct Machine {
    memory: [i32; 0],
    addr: usize,
    print_mode: PrintMode,
}

#[derive(Default)]
enum PrintMode {
    #[default]
    Decimal,
    ASCII,
}

impl PrintMode {
    fn print(&self, val: i32) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Decimal => print!("{val}"),
            Self::ASCII => print!("{}", c(val)?),
        };
        Ok(())
    }
}

fn c(val: i32) -> Result<char, Box<dyn Error>> {
    let val: u32 = u32::try_from(val)?;
    match char::from_u32(val) {
        Some(c) => Ok(c),
        None => Err(format!("can't make {val} into a char").into()),
    }
}

impl Machine {
    fn apply(&mut self, action: usize, arg: Option<usize>) -> Result<(), Box<dyn Error>> {
        match (action, arg) {
            // 1 - Set memory of address.
            (1, Some(val)) => self.memory[self.addr] = i32::try_from(val)?,
            // 2 - Change active memory address.
            (2, Some(addr)) => self.addr = addr,
            // 3 - Add value to address.
            (3, Some(val)) => self.memory[self.addr] += i32::try_from(val)?,
            // 4 - Subtract value from address.
            (4, Some(val)) => self.memory[self.addr] -= i32::try_from(val)?,
            // 5 - Multiply value by address.
            (5, Some(val)) => self.memory[self.addr] *= i32::try_from(val)?,
            // 6 - Divide address by value.
            (6, Some(val)) => self.memory[self.addr] /= i32::try_from(val)?,
            // 7 - Copy value at address to specified address.
            (7, Some(val)) => self.memory[val] = self.memory[self.addr],
            // 8 - Move value at address to specified address, set current address to 0.
            (8, Some(val)) => {
                self.memory[val] = self.memory[self.addr];
                self.memory[self.addr] = 0;
            }
            // 9 - Print the value of the current address to the console.
            (9, None) => self.print_mode.print(self.memory[self.addr])?,
            // 10 - Prints a line break into the console, moving all text to the next line.
            (10, None) => println!(),
            // 11 - Change print mode of console (0 for decimal/raw, 1 for ASCII).
            (11, Some(0)) => self.print_mode = PrintMode::Decimal,
            (11, Some(1)) => self.print_mode = PrintMode::ASCII,
            _ => {
                let x = Err(format!("unrecognized action {action} (arg = {arg:?})"));
                x?;
            }
        };
        Ok(())
    }
}

fn run(mut m: Machine, r: impl Read) -> Result<(), Box<dyn Error>> {
    let r = BufReader::new(r);
    let mut register = None;
    for l in r.lines() {
        match parse(&l?)? {
            Some(Syntax::Spaces(count)) => expect_none(register.replace(count))?,
            Some(Syntax::Tabs(count)) => m.apply(count, register.take())?,
            None => {}
        };
    }
    Ok(())
}

fn expect_none(o: Option<impl Debug>) -> Result<(), String> {
    match o {
        None => Ok(()),
        Some(x) => Err(format!("expected none but got some({x:?})")),
    }
}

enum Syntax {
    Spaces(usize),
    Tabs(usize),
}

fn parse(line: &str) -> Result<Option<Syntax>, String> {
    let mut spaces = 0;
    let mut tabs = 0;
    for c in line.chars() {
        match c {
            ' ' => spaces += 1,
            '\t' => tabs += 1,
            _ => (),
        };
    }
    match (spaces, tabs) {
        (0, 0) => Ok(None),
        (n, 0) => Ok(Some(Syntax::Spaces(n))),
        (0, n) => Ok(Some(Syntax::Tabs(n))),
        (a, b) => Err(format!(
            "error parsing {line:?}: {a} spaces and {b} tabs is illegal"
        )),
    }
}
