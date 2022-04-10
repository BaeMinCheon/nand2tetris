
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() < 2 {
        panic!("no argument !");
    } else {
        let filename: &str = arguments.get(1).unwrap();
        let contents: String = std::fs::read_to_string(filename).unwrap();
        let lines: Vec<String> = contents.split("\n").map(|s| s.to_string()).collect();
        let trimmed_lines = trim_raw_code(lines);
        let mut symbol_table = SymbolTable::Create();
        let mut address_to_save_symbol = 16;
        let mut lines_label_removed = vec![];
        {
            let mut indices_to_remove = std::collections::HashSet::new();
            let parser = Parser::Create(trimmed_lines.clone());
            let mut count = 0;
            while parser.hasMoreCommands() {
                let command_type = parser.commandType();
                match command_type {
                    Parser::CommandType::L_COMMAND => {
                        let symbol = parser.symbol();
                        let code_address = parser.getCurrentLineNumber() - count;
                        count += 1;
                        symbol_table.addEntry(symbol, code_address);
                        indices_to_remove.insert(parser.getCurrentLineNumber());
                    },
                    _ => {
                    }
                }
                parser.advance();
            }
            let mut index = 0;
            while index < trimmed_lines.len() {
                if indices_to_remove.contains(&index) == false {
                    lines_label_removed.push(trimmed_lines.get(index).unwrap().clone());
                }
                index += 1;
            }
        }
        let mut binaries = String::new();
        {
            let parser = Parser::Create(lines_label_removed);
            while parser.hasMoreCommands() {
                let command_type = parser.commandType();
                let binary = match command_type {
                    Parser::CommandType::A_COMMAND => {
                        let symbol = parser.symbol();
                        let symbol_string = if is_digit(&symbol) {
                            symbol
                        } else {
                            if symbol_table.contains(&symbol) {
                                let address = symbol_table.GetAddress(&symbol);
                                address.to_string()
                            } else {
                                let address = address_to_save_symbol.clone();
                                symbol_table.addEntry(symbol, address_to_save_symbol);
                                address_to_save_symbol += 1;
                                address.to_string()
                            }
                        };
                        let mut symbol_binary = digit_to_binary(symbol_string);
                        symbol_binary.insert(0, '0');
                        String::from(symbol_binary)
                    },
                    Parser::CommandType::C_COMMAND => {
                        let dest = parser.dest();
                        let dest_binary = Code::dest(&dest);
                        let comp = parser.comp();
                        let mut comp_binary = Code::comp(&comp);
                        let jump = parser.jump();
                        let jump_binary = Code::jump(&jump);
                        comp_binary.insert_str(0, "111");
                        comp_binary.push_str(dest_binary.as_str());
                        comp_binary.push_str(jump_binary.as_str());
                        String::from(comp_binary)
                    },
                    Parser::CommandType::L_COMMAND => {
                        let symbol = parser.symbol();
                        println!("[L_COMMAND] symbol: {}", symbol);
                        String::new()
                    },
                };
                binaries.push_str(binary.as_str());
                binaries.push('\n');
                parser.advance();
            }
            // Remove last new line character
            binaries.remove(binaries.len() - 1);
        }
        let mut output = std::fs::File::create("output.binary")?;
        output.write_all(binaries.as_bytes())?;
        Ok(())
    }
}

fn trim_raw_code(lines: Vec<String>) -> Vec<String> {
    let mut assembly_codes: Vec<String> = vec![];
    for line in lines {
        let first_split = line.split("//").nth(0).unwrap();
        let trimmed_line = first_split.trim();
        if trimmed_line.len() > 0 {
            assembly_codes.push(String::from(trimmed_line));
        }
    }
    assembly_codes
}

fn is_digit(input: &String) -> bool {
    let first_character = input.chars().nth(0).unwrap();
    ('0' <= first_character) && (first_character <= '9')
}

fn digit_to_binary(input: String) -> String {
    let mut result = String::new();
    let mut number: i32 = input.parse().unwrap();
    let mut divisor = i32::pow(2, 14);
    while divisor > 0 {
        if number / divisor == 1 {
            result.push('1');
        } else {
            result.push('0');
        }
        number %= divisor;
        divisor /= 2;
    }
    result
}

mod Parser {
    pub fn Create(lines: Vec<String>) -> Parser {
        Parser {
            lines: lines,
            current_index: std::cell::Cell::new(0),
        }
    }

    pub enum CommandType {
        A_COMMAND,
        C_COMMAND,
        L_COMMAND,
    }

    pub struct Parser {
        lines: Vec<String>,
        current_index: std::cell::Cell<usize>,
    }

    impl Parser {
        pub fn hasMoreCommands(&self) -> bool {
            self.current_index.get() < self.lines.len()
        }
        pub fn advance(&self) {
            self.current_index.set(self.current_index.get() + 1);
        }
        pub fn commandType(&self) -> CommandType {
            let current_command = self.lines.get(self.current_index.get()).unwrap();
            let first_character = current_command.chars().nth(0).unwrap();
            let result = match first_character {
                '@' => CommandType::A_COMMAND,
                '(' => CommandType::L_COMMAND,
                _ => CommandType::C_COMMAND,
            };
            result
        }
        pub fn symbol(&self) -> String {
            let result = match self.commandType() {
                CommandType::A_COMMAND => {
                    let current_command = self.lines.get(self.current_index.get()).unwrap();
                    String::from(&current_command.as_str()[1..])
                },
                CommandType::L_COMMAND => {
                    let current_command = self.lines.get(self.current_index.get()).unwrap();
                    let size = current_command.len();
                    String::from(&current_command.as_str()[1..(size - 1)])
                },
                _ => String::new(),
            };
            result
        }
        pub fn dest(&self) -> String {
            let result = match self.commandType() {
                CommandType::C_COMMAND => {
                    let current_command = self.lines.get(self.current_index.get()).unwrap();
                    let result = if current_command.contains("=") {
                        let mut splits = current_command.split("=");
                        String::from(splits.nth(0).unwrap())
                    } else {
                        String::new()
                    };
                    result
                },
                _ => String::new(),
            };
            result
        }
        pub fn comp(&self) -> String {
            let result = match self.commandType() {
                CommandType::C_COMMAND => {
                    let current_command = self.lines.get(self.current_index.get()).unwrap();
                    let result = if current_command.contains("=") && current_command.contains(";") {
                        let mut splits = current_command.split(|c| c == '=' || c == ';');
                        String::from(splits.nth(1).unwrap())
                    } else if current_command.contains("=") {
                        let mut splits = current_command.split('=');
                        String::from(splits.nth(1).unwrap())
                    } else if current_command.contains(";") {
                        let mut splits = current_command.split(';');
                        String::from(splits.nth(0).unwrap())
                    } else {
                        String::new()
                    };
                    result
                },
                _ => String::new(),
            };
            result
        }
        pub fn jump(&self) -> String {
            let result = match self.commandType() {
                CommandType::C_COMMAND => {
                    let current_command = self.lines.get(self.current_index.get()).unwrap();
                    let result = if current_command.contains(";") {
                        let mut splits = current_command.split(";");
                        String::from(splits.nth(1).unwrap())
                    } else {
                        String::new()
                    };
                    result
                },
                _ => String::new(),
            };
            result
        }
        pub fn getCurrentLineNumber(&self) -> usize {
            self.current_index.get()
        }
    }
}

mod Code {
    pub fn dest(input: &String) -> String {
        let result = match input.as_str() {
            "null0"     => "000",
            "M"         => "001",
            "D"         => "010",
            "MD"        => "011",
            "A"         => "100",
            "AM"        => "101",
            "AD"        => "110",
            "AMD"       => "111",
            _           => "000",
        };
        String::from(result)
    }
    pub fn comp(input: &String) -> String {
        let result = match input.as_str() {
            "0"     =>    "0101010",
            "1"     =>    "0111111",
            "-1"    =>    "0111010",
            "D"     =>    "0001100",
            "A"     =>    "0110000",
            "!D"    =>    "0001101",
            "!A"    =>    "0110001",
            "-D"    =>    "0001111",
            "-A"    =>    "0110011",
            "D+1"   =>    "0011111",
            "A+1"   =>    "0110111",
            "D-1"   =>    "0001110",
            "A-1"   =>    "0110010",
            "D+A"   =>    "0000010",
            "D-A"   =>    "0010011",
            "A-D"   =>    "0000111",
            "D&A"   =>    "0000000",
            "D|A"   =>    "0010101",
            "M"     =>    "1110000",
            "!M"    =>    "1110001",
            "-M"    =>    "1110011",
            "M+1"   =>    "1110111",
            "M-1"   =>    "1110010",
            "D+M"   =>    "1000010",
            "D-M"   =>    "1010011",
            "M-D"   =>    "1000111",
            "D&M"   =>    "1000000",
            "D|M"   =>    "1010101",
            _ => "",
        };
        String::from(result)
    }
    pub fn jump(input: &String) -> String {
        let result = match input.as_str() {
            "null"  => "000",
            "JGT"   => "001",
            "JEQ"   => "010",
            "JGE"   => "011",
            "JLT"   => "100",
            "JNE"   => "101",
            "JLE"   => "110",
            "JMP"   => "111",
            _       => "000",
        };
        String::from(result)
    }
}

mod SymbolTable {
    pub fn Create() -> SymbolTable {
        let table = std::collections::HashMap::from([
            (String::from("SP"),      0),
            (String::from("LCL"),     1),
            (String::from("ARG"),     2),
            (String::from("THIS"),    3),
            (String::from("THAT"),    4),
            (String::from("R0"),      0),
            (String::from("R1"),      1),
            (String::from("R2"),      2),
            (String::from("R3"),      3),
            (String::from("R4"),      4),
            (String::from("R5"),      5),
            (String::from("R6"),      6),
            (String::from("R7"),      7),
            (String::from("R8"),      8),
            (String::from("R9"),      9),
            (String::from("R10"),     10),
            (String::from("R11"),     11),
            (String::from("R12"),     12),
            (String::from("R13"),     13),
            (String::from("R14"),     14),
            (String::from("R15"),     15),
            (String::from("SCREEN"),  16384),
            (String::from("KBD"),     24576),
        ]);
        SymbolTable {
            table: table,
        }
    }

    pub struct SymbolTable {
        table: std::collections::HashMap<String, usize>,
    }

    impl SymbolTable {
        pub fn addEntry(&mut self, symbol: String, address: usize) {
            self.table.insert(symbol, address);
        }
        pub fn contains(&self, symbol: &String) -> bool {
            self.table.contains_key(symbol.as_str())
        }
        pub fn GetAddress(&self, symbol: &String) -> usize {
            self.table.get(symbol.as_str()).unwrap().clone()
        }
    }
}