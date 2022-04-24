
pub enum CommandType {
    C_ARITHMETIC,
    C_PUSH,
    C_POP,
    C_LABEL,
    C_GOTO,
    C_IF,
    C_FUNCTION,
    C_RETURN,
    C_CALL,
}

pub struct Parser {
    lines: Vec<String>,
    current_index: usize,
}

impl Parser {
    pub fn create(contents: String) -> Parser {
        let mut lines = contents.split("\n").map(|s| s.to_string()).collect();
        lines = Self::get_codes_comment_removed(&lines);
        Parser {
            lines,
            current_index: 0,
        }
    }
    pub fn hasMoreCommands(&self) -> bool {
        self.current_index < self.lines.len()
    }
    pub fn advance(&mut self) {
        self.current_index += 1;
    }
    pub fn commandType(&self) -> CommandType {
        let current_line = self.lines.get(self.current_index).unwrap();
        let first_word = current_line.split(' ').nth(0).unwrap();
        match first_word {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => CommandType::C_ARITHMETIC,
            "push" => CommandType::C_PUSH,
            "pop" => CommandType::C_POP,
            _ => panic!("invalid command type !"),
        }
    }
    pub fn arg0(&self) -> String {
        let current_line = self.lines.get(self.current_index).unwrap();
        let first_word = current_line.split(' ').nth(0).unwrap();
        String::from(first_word)
    }
    pub fn arg1(&self) -> String {
        let current_line = self.lines.get(self.current_index).unwrap();
        let second_word = current_line.split(' ').nth(1).unwrap();
        String::from(second_word)
    }
    pub fn arg2(&self) -> i32 {
        let current_line = self.lines.get(self.current_index).unwrap();
        let third_word = current_line.split(' ').nth(2).unwrap();
        third_word.parse::<i32>().unwrap()
    }
    fn get_codes_comment_removed(lines: &Vec<String>) -> Vec<String> {
        let mut vm_codes = vec![];
        for line in lines {
            let first_split = line.split("//").nth(0).unwrap();
            let trimmed_line = first_split.trim();
            if trimmed_line.len() > 0 {
                vm_codes.push(String::from(trimmed_line));
            }
        }
        vm_codes
    }
}