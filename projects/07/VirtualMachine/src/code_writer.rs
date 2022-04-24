
use super::parser;

pub struct CodeWriter {
    filename_output: String,
    contents: String,
    label_count: usize,
    filename_vm: String,
}

impl CodeWriter {
    pub fn create(filename: String) -> CodeWriter {
        CodeWriter {
            filename_output: filename,
            contents: String::new(),
            label_count: 0,
            filename_vm: String::new(),
        }
    }
    pub fn setFileName(&mut self, filename: &String) {
        self.filename_vm = Self::get_vm_filename_from_full_path(filename);
    }
    pub fn writeArithmetic(&mut self, command: &str) {
        match command {
            "add" => self.arithmetic_add(),
            "sub" => self.arithmetic_sub(),
            "neg" => self.arithmetic_neg(),
            "eq" => self.arithmetic_eq(),
            "gt" => self.arithmetic_gt(),
            "lt" => self.arithmetic_lt(),
            "and" => self.arithmetic_and(),
            "or" => self.arithmetic_or(),
            "not" => self.arithmetic_not(),
            _ => panic!("invalid command !"),
        }
    }
    pub fn writePushPop(&mut self, command: parser::CommandType, segment: &str, index: i32) {
        match command {
            parser::CommandType::C_PUSH => self.stack_push(segment, index),
            parser::CommandType::C_POP => self.stack_pop(segment, index),
            _ => panic!("invalid command !"),
        }
    }
    pub fn close(&self) -> std::io::Result<()> {
        let mut buffer = std::fs::File::create(&self.filename_output)?;
        std::io::Write::write_all(&mut buffer, self.contents.as_bytes());
        Ok(())
    }
    fn get_vm_filename_from_full_path(path: &String) -> String {
        let mut split = path.split("/").last().unwrap();
        split = split.split("\\").last().unwrap();
        split = split.split(".vm").nth(0).unwrap();
        String::from(split)
    }
}

impl CodeWriter {
    //          (A: ?) (D: ?) (RAM[0]: ?)
    // @SP      : Put SP into register A, which is 0(=value of SP). Let us suppose current RAM[0] is X.
    //          (A: 0) (D: ?) (RAM[0]: X)
    // AM=M-1
    // - A=M-1 : M is equal to X, which is address indicates current stack element. Plus, M-1 is equal to X-1, which is address indicates previous stack element. So, finally put X-1 into register A.
    // - M=M-1 : Subtract RAM[0] by 1. Now RAM[SP] will indicate previous stack element. RAM[0] will be turned X into X-1.
    //          (A: X-1) (D: ?) (RAM[0]: X-1)
    // D=M      : Because of A=M-1, M will return RAM[X-1] from now on. So, finally put RAM[X-1] into register D.
    //          (A: X-1) (D: RAM[X-1]) (RAM[0]: X-1)
    // @SP      : Put SP into register A.
    //          (A: 0) (D: RAM[X-1]) (RAM[0]: X-1)
    // AM=M-1
    // - A=M-1 : Because of M=M-1, M is equal to X-1. So finally put X-2 into register A.
    // - M=M-1 : Subtract RAM[0] by 1. RAM[0] will be turned X-1 into X-2.
    //          (A: X-2) (D: RAM[X-1]) (RAM[0]: X-2)
    // M=D+M    : D+M is equal to RAM[X-1]+RAM[X-2]. So finally put RAM[X-1]+RAM[X-2] into RAM[X-2].
    //          (A: X-2) (D: RAM[X-1]) (RAM[0]: X-2)
    // @SP      : Put SP into register A.
    //          (A: 0) (D: RAM[X-1]) (RAM[0]: X-2)
    // M=M+1    : Add RAM[0] by 1. RAM[0] will be turned X-2 into X-1.
    //          (A: 0) (D: RAM[X-1]) (RAM[0]: X-1)
    fn arithmetic_add(&mut self) {
        self.contents.push_str("
@SP
AM=M-1
D=M
@SP
AM=M-1
M=D+M
@SP
M=M+1
        ");
    }
    fn arithmetic_sub(&mut self) {
        self.contents.push_str("
@SP
AM=M-1
D=M
@SP
AM=M-1
M=M-D
@SP
M=M+1
        ");
    }
    fn arithmetic_neg(&mut self) {
        self.contents.push_str("
@SP
A=M-1
M=-M
        ");
    }
    fn arithmetic_eq(&mut self) {
        let number = self.label_count;
        self.contents.push_str(format!("
@SP
AM=M-1
D=M
@SP
A=M-1
D=M-D
M=-1
@comp_{number}
D;JEQ
@SP
A=M-1
M=0
(comp_{number})
        ").as_str());
        self.label_count += 1;
    }
    fn arithmetic_gt(&mut self) {
        let number = self.label_count;
        self.contents.push_str(format!("
@SP
AM=M-1
D=M
@SP
A=M-1
D=M-D
M=-1
@comp_{number}
D;JGT
@SP
A=M-1
M=0
(comp_{number})
        ").as_str());
        self.label_count += 1;
    }
    fn arithmetic_lt(&mut self) {
        let number = self.label_count;
        self.contents.push_str(format!("
@SP
AM=M-1
D=M
@SP
A=M-1
D=M-D
M=-1
@comp_{number}
D;JLT
@SP
A=M-1
M=0
(comp_{number})
        ").as_str());
        self.label_count += 1;
    }
    fn arithmetic_and(&mut self) {
        self.contents.push_str("
@SP
AM=M-1
D=M
@SP
A=M-1
M=D&M
        ");
    }
    fn arithmetic_or(&mut self) {
        self.contents.push_str("
@SP
AM=M-1
D=M
@SP
A=M-1
M=D|M
        ");
    }
    fn arithmetic_not(&mut self) {
        self.contents.push_str("
@SP
A=M-1
M=!M
        ");
    }
}

impl CodeWriter {
    fn stack_push(&mut self, segment: &str, index: i32) {
        match segment {
            "local" => self.stack_push_local(index),
            "argument" => self.stack_push_argument(index),
            "this" => self.stack_push_this(index),
            "that" => self.stack_push_that(index),
            "pointer" => self.stack_push_pointer(index),
            "temp" => self.stack_push_temp(index),
            "constant" => self.stack_push_constant(index),
            "static" => self.stack_push_static(index),
            _ => panic!("invalid command !"),
        }
    }
    //          (A: ?) (D: ?)
    // @{index} : Put index into register A.
    //          (A: index) (D: ?)
    // D=A      : Put index into register D.
    //          (A: index) (D: index)
    // @LCL     : Put 1(=value of LCL) into register A.
    //          (A: 1) (D: index)
    // A=M+D    : M+D is equal to RAM[1]+index. Put RAM[1]+index into register A, which indicates address of local segment.
    //          (A: RAM[1]+index) (D: index)
    // D=M      : Put RAM[RAM[1]+index] into register D, which is value of local segment.
    //          (A: RAM[1]+index) (D: RAM[RAM[1]+index])
    // @SP      : Put 0(=value of SP) into register A.
    //          (A: 0) (D: RAM[RAM[1]+index])
    // A=M      : Put RAM[0] into register A.
    //          (A: RAM[0]) (D: RAM[RAM[1]+index])
    // M=D      : Put RAM[RAM[1]+index] into RAM[RAM[0]]. So, the value is pushed successfully.
    //          (A: RAM[0]) (D: RAM[RAM[1]+index])
    // @SP      : Put 0 into register A.
    //          (A: 0) (D: RAM[RAM[1]+index])
    // M=M+1    : Put RAM[0]+1 into RAM[0] to move SP.
    //          (A: 0) (D: RAM[RAM[1]+index])
    fn stack_push_local(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@LCL
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_argument(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@ARG
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_this(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@THIS
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_that(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@THAT
A=M+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_pointer(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@3
A=A+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_temp(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@5
A=A+D
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_constant(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
    fn stack_push_static(&mut self, index: i32) {
        let vm_name = self.filename_vm.clone();
        self.contents.push_str(format!("
@{vm_name}.{index}
D=M
@SP
A=M
M=D
@SP
M=M+1
        ").as_str());
    }
}

impl CodeWriter {
    fn stack_pop(&mut self, segment: &str, index: i32) {
        match segment {
            "local" => self.stack_pop_local(index),
            "argument" => self.stack_pop_argument(index),
            "this" => self.stack_pop_this(index),
            "that" => self.stack_pop_that(index),
            "pointer" => self.stack_pop_pointer(index),
            "temp" => self.stack_pop_temp(index),
            "constant" => self.stack_pop_constant(index),
            "static" => self.stack_pop_static(index),
            _ => panic!("invalid command !"),
        }
    }
    //          (A: ?) (D: ?)
    // @{index} : Put index into register A.
    //          (A: index) (D: ?)
    // D=A      : Put index into register D.
    //          (A: index) (D: index)
    // @LCL     : Put 1(=value of LCL) into register A.
    //          (A: 1) (D: index)
    // D=M+D    : M+D is equal to RAM[1]+index. Put it into register D, which is address of local segment.
    //          (A: 1) (D: RAM[1]+index)
    // @R13     : Put 13(=value of R13) into register A.
    //          (A: 13) (D: RAM[1]+index)
    // M=D      : Put RAM[1]+index into RAM[13]. For storing address temporarily.
    //          (A: 13) (D: RAM[1]+index)
    // @SP      : Put 0(=value of SP) into register A.
    //          (A: 0) (D: RAM[1]+index)
    // AM=M-1
    // - A=M-1  : Suppose current RAM[0] as X. Then, it puts X-1 into register A, which is address indicates previous stack element.
    // - M=M-1  : Subtract RAM[0] by 1. Now RAM[SP] will indicate previous stack element.
    //          (A: X-1) (D: RAM[1]+index)
    // D=M      : Put RAM[X-1] into register D.
    //          (A: X-1) (D: RAM[X-1])
    // @R13     : Put 13 into register A.
    //          (A: 13) (D: RAM[X-1])
    // A=M      : Put RAM[13](=RAM[1]+index) into register A.
    //          (A: RAM[1]+index) (D: RAM[X-1])
    // M=D      : Put RAM[X-1] into RAM[RAM[1]+index]. The local segment whose location is RAM[1]+index will be updated.
    //          (A: RAM[1]+index) (D: RAM[X-1])
    fn stack_pop_local(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@LCL
D=M+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_argument(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@ARG
D=M+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_this(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@THIS
D=M+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_that(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@THAT
D=M+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_pointer(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@3
D=A+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_temp(&mut self, index: i32) {
        self.contents.push_str(format!("
@{index}
D=A
@5
D=A+D
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D
        ").as_str());
    }
    fn stack_pop_constant(&mut self, index: i32) {
        panic!("[pop constant] cannot be executed as constant is not stored in memory");
    }
    fn stack_pop_static(&mut self, index: i32) {
        let vm_name = self.filename_vm.clone();
        self.contents.push_str(format!("
@SP
AM=M-1
D=M
@{vm_name}.{index}
M=D
        ").as_str());
    }
}