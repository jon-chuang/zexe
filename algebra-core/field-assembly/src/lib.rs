extern crate std;
extern crate regex;
use regex::Regex;

const MAX_REGS: usize = 6;

// Only works for up to
pub fn generate_macro_string (num_limbs:usize) -> std::string::String {
    if (num_limbs > 3 * MAX_REGS) {//|| (MAX_REGS < 6) {
        panic!("Number of limbs must be <= {} and MAX_REGS >= 6", 3*MAX_REGS);
    }
    let mut macro_string = String::from(
    "macro_rules! asm_mul {
        ($limbs:expr, $a:expr, $b:expr, $modulus:expr, $inverse:expr) => {
            match $limbs {");
    macro_string = generate_matches(num_limbs, macro_string, true);

    macro_string = format!("{}{}", macro_string,
    "macro_rules! asm_square {
        ($limbs:expr, $a:expr, $modulus:expr, $inverse:expr) => {
            match $limbs {");
    macro_string = generate_matches(num_limbs, macro_string, false);
    macro_string
}

fn generate_matches (num_limbs: usize, mut macro_string: String, is_mul: bool) -> String {
    for i in 2..(num_limbs+1) {
        let mut limb_specialisation = format!("
                {} => {{", i);
        // logic to format macro based on how many limbs there are, whether it is a mul
        let (mut b_declare, mut spills_declare, mut b, mut spills) = ("                   // $3", String::from(""), "$0", "");
        let mut rs_clobber = String::from("");
        for k in 0..std::cmp::min(i, MAX_REGS) { rs_clobber = format!("{}{}", rs_clobber, format!("\"r{}\", ", 8+k)); }
        if is_mul {
            b_declare = ",                  // $3
                              \"r\"(&$b)";
            b = "$4";
            spills_declare = String::from("                        // $4");
        }
        if i > MAX_REGS {
            let extra_reg = if i <= 2*MAX_REGS { 2*(i-MAX_REGS) } else { i };
            limb_specialisation = format!("{}{}", limb_specialisation, format!("
                    let mut spills = [0u64; {}];", extra_reg));
            if is_mul { spills = "$5"; } else { spills = "$4";}
            spills_declare = format!(",                       // ${}
                              \"r\"(&mut spills)                // {}", 3+(is_mul as usize), spills);
        }

        // Actual asm declaration
        limb_specialisation = format!("{}{}", limb_specialisation, format!("
                    unsafe {{
                        asm!({asm_string}
                            :
                            : \"r\"(&mut $a),                   // $0
                              \"r\"(&$modulus),                 // $1
                              \"i\"(0u64),                      // $2
                              \"i\"($inverse){b_declare}{spills_declare}
                            : \"rcx\", \"rbx\", \"rdx\", \"rax\", {rs_clobber}\"cc\", \"memory\"
                        );
                    }}
                }}",
                asm_string = transform_asm_mul_string(i, generate_asm_mul_string(i, "$0", b), spills, "$0"),
                rs_clobber=rs_clobber,
                b_declare=b_declare,
                spills_declare=spills_declare));
        macro_string = format!("{}{}", macro_string, limb_specialisation);
    }
    macro_string = format!("{}{}", macro_string, format!("
            x => panic!(\"asm_mul (no-carry): number of limbs supported is 2 up to {}. You had {{}}.\", x)
        }};
    }}
}}

", num_limbs));
    macro_string
}

fn generate_asm_mul_string (limbs: usize, a: &str, b: &str) -> String {
    let mut asm_string = String::from("");
    for i in 0..limbs {
        // First inner loop
        if i == 0 {
            asm_string = format!("{}{}", asm_string,format!("\"
                            movq 0({a}), %rdx
                            xorq %rcx, %rcx
                                mulxq 0({b}), %r8, %r9",
                                a=a, b=b));
            for j in 1..limbs-1 {
                asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}({b}), %rax, %r{}
                                adcxq %rax, %r{}",
                                j*8, 8 + ((j+1) % limbs), 8+j, b=b));
            }
            asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}({b}), %rax, %rcx
                                mov $2, %rbx
                                adcxq %rax, %r{}
                                adcxq %rbx, %rcx               // %rcx is carry1",
                                (limbs-1)*8, 8+limbs-1, b=b));
        } else {
            asm_string = format!("{}{}", asm_string, format!("
                            movq {}($0), %rdx", i * 8));
            for j in 0..limbs-1 {
                asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}({b}), %rax, %rbx
                                adcxq %rax, %r{}
                                adoxq %rbx, %r{}",
                                j * 8, 8 + ((j+i) % limbs), 8 + ((j+i+1) % limbs), b=b));
                }
            asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}({b}), %rax, %rcx
                                mov $2, %rbx
                                adcxq %rax, %r{}
                                adoxq %rbx, %rcx
                                adcxq %rbx, %rcx",
                                (limbs-1) * 8,
                                8 + ((i+limbs-1) % limbs),
                                b=b));
        }
        // Second inner loop
        asm_string = format!("{}{}", asm_string, format!("
                            movq $3, %rdx
                            mulxq %r{}, %rdx, %rax            // wrapping_mul", 8+i));
        asm_string = format!("{}{}", asm_string, format!("
                                mulxq 0($1), %rax, %rbx
                                adcxq %r{}, %rax              // put junk in rax
                                adoxq %rbx, %r{}",
                                8 + (i % limbs),
                                8 + ((i+1) % limbs)));
        for j in 1..limbs-1 {
            asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}($1), %rax, %rbx
                                adcxq %rax, %r{}
                                adoxq %rbx, %r{}",
                                j * 8,
                                8 + ((j+i) % limbs),
                                8 + ((j+i+1) % limbs)));
        }
        asm_string = format!("{}{}", asm_string, format!("
                                mulxq {}($1), %rax, %r{2}
                                mov $2, %rbx
                                adcxq %rax, %r{}
                                adoxq %rcx, %r{2}
                                adcxq %rbx, %r{2}",
                                (limbs-1)*8,
                                8 + ((i+limbs-1) % limbs),
                                8 + ((i) % limbs)));
    }
    for i in 0..limbs {
        asm_string = format!("{}{}", asm_string, format!("
                            movq %r{}, {}($0)", 8+(i % limbs), i*8));
    }
    format!("{}{}", asm_string, "
                        \"")
}


fn get_registers (limbs: usize) -> (usize, Vec<Vec<usize>>) {
    assert!(limbs <= 3*MAX_REGS);

    if limbs <= MAX_REGS {
        (0, Vec::new())
    } else if limbs == MAX_REGS + 1 {
        (1, vec![
                vec![MAX_REGS/2, MAX_REGS]
                ])
    } else if limbs == MAX_REGS + 2 {
        (2, vec![
                vec![MAX_REGS/2, MAX_REGS],
                vec![MAX_REGS/2+1, MAX_REGS+1]
                ])
    } else if limbs == MAX_REGS + 3 {
        (3, vec![
                vec![MAX_REGS/2, MAX_REGS],
                vec![MAX_REGS/2+1, MAX_REGS+1],
                vec![MAX_REGS/2+2, MAX_REGS+2]
                ])
    } else if limbs <= MAX_REGS * 2 {
        let n_spills = limbs - MAX_REGS;
        let mut values = Vec::new();
        for i in 0..n_spills {
            values.push(vec![i, MAX_REGS+i]);
        }
        (n_spills, values)
    } else { // (if limbs <= MAX_REGS * 3)
        let mut values = Vec::new();
        for i in 0..MAX_REGS {
            if i < limbs - 2*MAX_REGS {
                values.push(vec![i, MAX_REGS+i, 2*MAX_REGS+i]);
            } else {
                values.push(vec![i, MAX_REGS+i]);
            }
        }
        (MAX_REGS, values)
    }
}

// This is a compilation pass which converts abstract
// register numbers into x64 registers with spills.
// Rather hacky at this stage
fn transform_asm_mul_string (limbs: usize, asm_string: String, spills: &str, a: &str) -> String {
    // println!("{}", asm_string);
    let (n_spills, spillable) = get_registers(limbs);
    let mut lines = asm_string.split("\n");

    let re = Regex::new(r"%r\d+").unwrap();
    let number = Regex::new(r"\d+").unwrap();

    let mut line_number = 0;
    let mut reg_sequence: Vec<Vec<(usize, usize)>> = std::iter::repeat(vec![]).take(n_spills).collect::<Vec<_>>();

    let mut edited_lines: Vec<String> = Vec::new();

    for line in lines {
        edited_lines.push(line.to_string());
        line_number += 1;
        if re.is_match(&line.to_string()) {
            let words = line.split(" ");
            for word in words {
                if re.is_match(&word.to_string()) {
                    let num = number.captures(word).unwrap();
                    let reg_num = &num[0].parse::<usize>().unwrap();
                    for i in 0..n_spills {
                        if spillable[i].contains(&(*reg_num-8)) {
                            reg_sequence[i].push((line_number, *reg_num-8));
    }    }    }    }    }    }

    let mut swap_sequence: Vec<Vec<(usize, usize, usize)>> = std::iter::repeat(vec![]).take(n_spills).collect::<Vec<_>>();
    for i in 0..n_spills {
        let length = reg_sequence[i].len();
        if length > 0 {
            for j in 0..reg_sequence[i].len()-1 {
                if reg_sequence[i][j].1 != reg_sequence[i][j+1].1 {
                    swap_sequence[i].push((reg_sequence[i][j].0,        // line number
                                           reg_sequence[i][j].1,        // current reg index
                                           reg_sequence[i][j+1].1));    // next reg index
                }
            }
        swap_sequence[i].push((reg_sequence[i][length-1].0,
                               reg_sequence[i][length-1].1,
                               reg_sequence[i][length-1].1));
    }
        let length = swap_sequence[i].len();
        if length > 1 && spillable[i].len() <= 2 {
            for j in 0..length {
                let swap = &swap_sequence[i][j];
                if j < length - 3 {
                    let index1 = if swap.1 >= MAX_REGS { n_spills + i } else { i };
                    let index2 =  if swap.2 >= MAX_REGS { n_spills + i } else { i };
                    edited_lines[swap.0-1] = format!("{}{}", edited_lines[swap.0-1], format!("
                                movq %r{reg}, {index1}({dest})
                                movq {index2}({spills}), %r{reg}",
                                reg=8+spillable[i][0], index1=index1*8, index2=index2*8,
                                dest=if j!=length-4 {spills} else {a}, spills=spills));
                }
            }
            let swap = &swap_sequence[i][length-3];
            let index1 = if swap.1 >= MAX_REGS { n_spills + i } else { i };
            edited_lines[swap.0-1] = format!("{}{}", edited_lines[swap.0-1], format!("
                                movq %r{reg}, {index1}({dest})",
                                reg=8+spillable[i][0], index1=index1*8, dest=a));
            edited_lines[&swap_sequence[i][length-2].0-1] = "".to_string();
            edited_lines[&swap_sequence[i][length-1].0-1] = "".to_string();
        } else {
            for j in 0..length {
                let swap = &swap_sequence[i][j];
                if j < length - 4 {
                    edited_lines[swap.0-1] = format!("{}{}", edited_lines[swap.0-1], format!("
                                movq %r{reg}, {index1}({dest})
                                movq {index2}({spills}), %r{reg}",
                                reg=8+spillable[i][0], index1=swap.1*8, index2=swap.2*8,
                                dest=if j!=length-5 && j!=length-6 {spills} else {a}, spills=spills));
                }
            }
            let swap = &swap_sequence[i][length-4];
            edited_lines[swap.0-1] = format!("{}{}", edited_lines[swap.0-1], format!("
                                movq %r{reg}, {index1}({dest})",
                                reg=8+spillable[i][0], index1=swap.1*8, dest=a));
            edited_lines[&swap_sequence[i][length-3].0-1] = "".to_string();
            edited_lines[&swap_sequence[i][length-2].0-1] = "".to_string();
            edited_lines[&swap_sequence[i][length-1].0-1] = "".to_string();
        }
    }
    let length = edited_lines.len();
    for i in 0..limbs+1 {
        if edited_lines[length-1-i] == "" {
            edited_lines.remove(length-1-i);
        }
    }
    let mut interspersed = edited_lines[..].join("\n");
    for i in 0..n_spills {
        interspersed = interspersed.replace(&format!("%r{}", 8+spillable[i][1]), &format!("%r{}", 8+spillable[i][0]));
        if spillable[i].len() == 3 {
            interspersed = interspersed.replace(&format!("%r{}", 8+spillable[i][2]), &format!("%r{}", 8+spillable[i][0]));
        }
    }
    interspersed
}