use crate::ast::*;

use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

pub struct Compile {
    output_file: File,
}

impl Compile {
    pub fn new(output_path: &str) -> Compile {
        if !fs::metadata(output_path).is_ok() {
            match fs::create_dir(output_path) {
                Ok(_) => {}
                Err(e) => panic!("Error Creating file target {}", e),
            }
        } else {
            if fs::metadata(format!("{}/{}", output_path, "output.asm")).is_ok() {
                match fs::remove_file(format!("{}/{}", output_path, "output.asm")) {
                    Err(e) => {
                        panic!("Error While Deleting Prev File {}", e)
                    }
                    Ok(_) => {}
                }
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/{}", output_path, "output.asm"));
        match file {
            Ok(file) => {
                let mut comp = Compile { output_file: file };

                comp.init_global_func();
                comp
            }
            Err(e) => {
                panic!("Error caused while creating/ opening file {}", e)
            }
        }
    }

    pub fn write(&mut self, line_to_write: &str) {
        let line = writeln!(self.output_file, "{}", line_to_write);

        match line {
            Err(e) => {
                panic!("Error caused while writing to file with {}", e)
            }
            Ok(_) => {}
        }
    }

    fn init_global_func(&mut self) {
        self.write("BITS 64");
        self.write("segment .text");
        self.write("dump:");
        self.write("    mov     r9, -3689348814741910323");
        self.write("    sub     rsp, 40");
        self.write("    mov     BYTE [rsp+31], 10");
        self.write("    lea     rcx, [rsp+30]");
        self.write(".L2:");
        self.write("    mov     rax, rdi");
        self.write("    lea     r8, [rsp+32]");
        self.write("    mul     r9");
        self.write("    mov     rax, rdi");
        self.write("    sub     r8, rcx");
        self.write("    shr     rdx, 3");
        self.write("    lea     rsi, [rdx+rdx*4]");
        self.write("    add     rsi, rsi");
        self.write("    sub     rax, rsi");
        self.write("    add     eax, 48");
        self.write("    mov     BYTE [rcx], al");
        self.write("    mov     rax, rdi");
        self.write("    mov     rdi, rdx");
        self.write("    mov     rdx, rcx");
        self.write("    sub     rcx, 1");
        self.write("    cmp     rax, 9");
        self.write("    ja      .L2");
        self.write("    lea     rax, [rsp+32]");
        self.write("    mov     edi, 1");
        self.write("    sub     rdx, rax");
        self.write("    xor     eax, eax");
        self.write("    lea     rsi, [rsp+32+rdx]");
        self.write("    mov     rdx, r8");
        self.write("    mov     rax, 1");
        self.write("    syscall");
        self.write("    add     rsp, 40");
        self.write("    ret");
        self.write("global _start");
        self.write("_start:");
    }

    fn assemble_push(&mut self, number: impl std::fmt::Display) {
        self.write("    ;; -- push  --");
        self.write(&*format!("    push {}", number))
    }

    fn assemble_plus(&mut self) {
        self.write("    ;; -- plus --");
        self.write("    pop rax");
        self.write("    pop rbx");
        self.write("    add rax, rbx");
        self.write("    push rax")
    }

    fn assemble_minus(&mut self) {
        self.write("    ;; -- minus --");
        self.write("    pop rax");
        self.write("    pop rbx");
        self.write("    sub rbx, rax");
        self.write("    push rbx")
    }

    fn assemble_multiply(&mut self) {
        self.write("    ;; -- multiply --");
        self.write("    pop rax");
        self.write("    pop rbx");
        self.write("    mul rbx");
        self.write("    push rax")
    }

    fn assemble_function(&mut self, name: String) {
        match name.as_ref() {
            "print" => {
                self.write("    ;; -- dump --");
                self.write("    pop rdi");
                self.write("    call dump") 
            }
            _ => {}
        }
    }

    fn assemble_equal(&mut self){
        self.write("    ;; -- equal --");
        self.write("    mov rcx, 0");
        self.write("    mov rdx, 1");
        self.write("    pop rax");
        self.write("    pop rbx");
        self.write("    cmp rax, rbx");
        self.write("    cmove rcx, rdx");
        self.write("    push rcx");
    }

    fn evaluate_both_sides(&mut self, args: Vec<Expr>) {
        self.evaluate(args[0].clone());
        self.evaluate(args[1].clone());
    }

    pub fn evaluate(&mut self, expr: Expr) {
        match expr {
            Expr::OpExpr(bx_expr) => match *bx_expr {
                OpExpr { op, args } => match op {
                    Operator::Plus => {
                        self.evaluate_both_sides(args);

                        self.assemble_plus()
                    }
                    Operator::Substract => {
                        self.evaluate_both_sides(args);

                        self.assemble_minus()
                    }
                    Operator::Multiply => {
                        self.evaluate_both_sides(args);

                        self.assemble_multiply()
                    }
                    Operator::Equal => {
                        self.evaluate_both_sides(args);

                        self.assemble_equal()
                    }
                    Operator::Call(name) => {
                        self.evaluate(args[0].clone());

                        self.assemble_function(name)
                    }
                    _ => {}
                },
            },
            Expr::OpLiteral(bx_lit) => match *bx_lit {
                Literal::Integer(int_val) => self.assemble_push(int_val),
                Literal::FloatingPoint(float_val) => self.assemble_push(float_val),
                _ => {}
            },
        }
    }

    pub fn execute(&mut self) {
        self.write("    mov rax, 60");
        self.write("    mov rdi, 0");
        self.write("    syscall");

        let nasm_output = Command::new("nasm")
            .arg("-felf64")
            .arg("./output/output.asm")
            .output();

        match nasm_output {
            Ok(output) => {
                if output.status.success() {
                    let ld_output = Command::new("ld")
                        .arg("-o")
                        .arg("./output/output")
                        .arg("./output/output.o")
                        .output();

                    match ld_output {
                        Ok(output) => {
                            if output.status.success() {
                                let executable_output = Command::new("./output/output")
                                    .output()
                                    .expect("Failed to run the executable");

                                let stdout = String::from_utf8_lossy(&executable_output.stdout);

                                println!("{}", stdout);

                                if !executable_output.status.success() {
                                    eprintln!(
                                        "Executable failed with error: {:?}",
                                        executable_output.status
                                    );
                                }
                            } else {
                                eprintln!("ld command failed with error: {:?}", output.status);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error executing ld command: {:?}", err);
                        }
                    }
                } else {
                    eprintln!("nasm command failed with error: {:?}", output.status);
                }
            }
            Err(err) => {
                eprintln!("Error executing nasm command: {:?}", err);
            }
        }
    }
}
