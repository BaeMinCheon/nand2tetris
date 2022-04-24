fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() < 2 {
        panic!("no argument !");
    } else {
        let path: &str = arguments.get(1).unwrap();
        let virtual_machine = VirtualMachine::create(String::from(path));
        virtual_machine.run();
    }
}

mod parser;
mod code_writer;

struct VirtualMachine {
    path_of_files: Vec<String>,
}

impl VirtualMachine {
    fn create(path: String) -> VirtualMachine {
        let std_path = std::path::Path::new(&path);
        let is_directory = std_path.is_dir();
        let mut path_of_files = vec![];
        if is_directory {
            for entry in std_path.read_dir().unwrap() {
                let pathbuf = entry.unwrap().path();
                let path = pathbuf.into_os_string().into_string().unwrap();
                if Self::validate_file_extension(&path) {
                    path_of_files.push(path);
                } else {
                    // Do nothing
                }
            }
        } else {
            if Self::validate_file_extension(&path) {
                path_of_files.push(path);
            } else {
                panic!("the extension of file is not .vm !");
            }
        }
        VirtualMachine {
            path_of_files,
        }
    }
    fn run(&self) {
        let mut writer = code_writer::CodeWriter::create(String::from("output.asm"));
        for path_of_file in &self.path_of_files {
            let contents = std::fs::read_to_string(path_of_file).unwrap();
            let mut parser = parser::Parser::create(contents);
            let filename_vm = writer.setFileName(path_of_file);
            while parser.hasMoreCommands() {
                match parser.commandType() {
                    parser::CommandType::C_ARITHMETIC => {
                        writer.writeArithmetic(parser.arg0().as_str());
                    },
                    parser::CommandType::C_PUSH => {
                        writer.writePushPop(parser::CommandType::C_PUSH, parser.arg1().as_str(), parser.arg2());
                    },
                    parser::CommandType::C_POP => {
                        writer.writePushPop(parser::CommandType::C_POP, parser.arg1().as_str(), parser.arg2());
                    },
                    _ => {
                        panic!("invalid command type !");
                    }
                }
                parser.advance();
            }
        }
        writer.close().unwrap();
    }
    fn validate_file_extension(filename: &String) -> bool {
        let mut result = false;
        let splits: Vec<String> = String::from(filename).split(".").map(|s| s.to_string()).collect();
        if splits.len() > 1 {
            let file_extension = splits.last().unwrap();
            if file_extension == "vm" {
                result = true;
            }
        }
        result
    }
}