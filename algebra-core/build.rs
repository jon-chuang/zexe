use std::env;
use std::fs;
use std::path::Path;

const MAX_LIMBS: usize = 8;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("assembly.rs");

    let macro_string = generate_macro_string(MAX_LIMBS);

    fs::write(
        &dest_path,
        macro_string
    ).unwrap();

    println!("cargo:rerun-if-changed=src/fields/assembly_string_gen.rs");
    println!("cargo:rerun-if-changed=build.rs");
}

// For now, generated code only works for up to  8/10 limbss
// In the future, we can try to implement data movement to and from an address
// for higher number of limbs
fn generate_asm_mul_string (limbs: usize) -> String {
    let mut asm_string = String::from("");
    for i in 0..limbs {
        if i == 0 {
            asm_string = format!("{}{}", asm_string,
                            "\"
                            xorq %rdi, %rdi
                                mulxq 0($2), %r8, %r9
                                mulxq 8($2), %rax, %r10
                                adcxq %rax, %r9
                                mulxq 16($2), %rax, %r11
                                adcxq %rax, %r10
                                mulxq 24($2), %rax, %rdi
                                adcxq %rax, %r11
                                adcxq $4, %rdi               // %rdi is carry1");
        } else {
            asm_string = format!("{}{}", asm_string, format!("
                            movq {}($1), %rdx", i * 8));
            for j in 0..limbs-1 {
                let temp_inner = format!("
                                mulxq {}($2), %rax, %rbx
                                adcxq %rax, %r{}
                                adoxq %rbx, %r{}",
                                ((j+i+1) % limbs) * 8,
                                8 + ((j+i+1) % limbs),
                                8 + ((j+i+2) % limbs));
                asm_string = format!("{}{}", asm_string, temp_inner);
            }
            asm_string = format!("{}{}", asm_string,"
                                adoxq %rbx, %rdi");
        }
        asm_string = format!("{}{}", asm_string, format!("
                            movq $5, %rdx
                            mulxq %r{}, %rdx, %rax            // wrapping_mul", 8+i));
        asm_string = format!("{}{}", asm_string, format!("
                                mulxq 0($3), %rax, %rbx
                                adcxq %r{}, %rax              // put junk in rax
                                adoxq %rbx, %r{}",
                                8 + ((i+1) % limbs),
                                8 + ((i+2) % limbs)));
        for j in 1..limbs {
            let temp_inner = format!("
                                mulxq {}($3), %rax, %rbx
                                adcxq %rax, %r{}
                                adoxq %rbx, %r{}",
                                j * 8,
                                8 + ((j+i+1) % limbs),
                                8 + ((j+i+2) % limbs));
            asm_string = format!("{}{}", asm_string, temp_inner);
        }
        asm_string = format!("{}{}", asm_string, format!("
                            adcxq %rbx, %r{}
                            adoxq %rdi, %r{}",
                            8 + ((i+limbs-1) % limbs),
                            8 + ((i+limbs-1) % limbs)));
    }
    for i in 0..limbs {
        asm_string = format!("{}{}", asm_string, format!("
                            movq %r{}, {}($0)", 8+((i+limbs-1) % limbs), i*8));
    }
    format!("{}{}",asm_string, "\"")
}

fn generate_macro_string (max_limbs:usize) -> std::string::String {
    let mut macro_string = String::from(
    "macro_rules! generate_asm_mul {
        ($limbs:expr) => {
            fn mul_asm (
                a: [u64; $limbs],
                b: [u64; $limbs],
                modulus: [u64; $limbs],
                inverse: u64
            ) -> [u64; $limbs] {
                let result = match $limbs {
    ");
    for i in 2..(max_limbs+1) {
        let mut rs = String::from("");
        for k in 0..i {
            rs = format!("{}{}", rs, format!("\"r{}\", ", 8+k));
        }
        let limb_specialisation = format!(
    "           {} => {{
                    const ZERO: u64 = 0;
                    let mut result = MaybeUninit::<[u64; $limbs]>::uninit();
                    unsafe {{
                        asm!({}
                            // :
                            // : \"r\"(result.as_mut_ptr()),
                            //   \"r\"(&a), \"r\"(&b),
                            //   \"m\"(ZERO),
                            //   \"m\"(modulus[0]),
                            //   \"m\"(modulus[1]),
                            //   \"m\"(modulus[2]),
                            //   \"m\"(modulus[3]),
                            //   \"m\"(inverse)
                            // : \"rdx\", \"rdi\", \"r8\", \"r9\", \"r10\", \"r11\", \"r12\", \"r13\", \"r14\", \"r15\", \"cc\", \"memory\"
                            : \"=r\"(result.as_mut_ptr())               // $0
                            : \"r\"(&a),                                // $1
                              \"r\"(&b),                                // $2
                              \"r\"(&modulus),                          // $3
                              \"m\"(ZERO),                              // $4
                              \"m\"(inverse)                            // $5
                            : \"rax\", \"rbx\", \"rdx\", \"rdi\", {} \"cc\", \"memory\"
                        );
                    }}
                    let r = unsafe {{ result.assume_init() }};
                    r
                }},

    ", i, generate_asm_mul_string(i), rs);//ASM_STR);//
        macro_string = format!("{}{}", macro_string, limb_specialisation);
    }
    macro_string = format!("{}{}", macro_string,
            "x => panic!(\"asm_mul (no-carry): number of limbs supported is 2 up to 8. You had {}\", x)
            };
            result
        };
    }
}");
    macro_string
}


const ASM_STR:&'static str = "
                            \"
                            // Assembly from Aztec's Barretenberg implementation, see
                            // <https://github.com/AztecProtocol/barretenberg/blob/master/src/barretenberg/fields/asm_macros.hpp>
                            movq 0($1), %rdx
                            xorq %r8, %r8

                            mulxq 0($2), %r13, %r14
                            mulxq 8($2), %r8, %r9
                            mulxq 16($2), %r15, %r10
                            mulxq 24($2), %rdi, %r12

                            movq %r13, %rdx
                            mulxq $8, %rdx, %r11
                            adcxq %r8, %r14
                            adoxq %rdi, %r10
                            adcxq %r9, %r15
                            adoxq $3, %r12
                            adcxq $3, %r10

                            mulxq $4, %r8, %r9
                            mulxq $5, %rdi, %r11
                            adoxq %r8, %r13
                            adcxq %rdi, %r14
                            adoxq %r9, %r14
                            adcxq %r11, %r15

                            mulxq $6, %r8, %r9
                            mulxq $7, %rdi, %r11
                            adoxq %r8, %r15
                            adcxq %rdi, %r10
                            adoxq %r9, %r10
                            adcxq %r11, %r12
                            adoxq $3, %r12

                            movq 8($1), %rdx
                            mulxq 0($2), %r8, %r9
                            mulxq 8($2), %rdi, %r11
                            adcxq %r8, %r14
                            adoxq %r9, %r15
                            adcxq %rdi, %r15
                            adoxq %r11, %r10

                            mulxq 16($2), %r8, %r9
                            mulxq 24($2), %rdi, %r13
                            adcxq %r8, %r10
                            adoxq %rdi, %r12
                            adcxq %r9, %r12
                            adoxq $3, %r13
                            adcxq $3, %r13

                            movq %r14, %rdx
                            mulxq $8, %rdx, %r8
                            mulxq $4, %r8, %r9
                            mulxq $5, %rdi, %r11
                            adoxq %r8, %r14
                            adcxq %rdi, %r15
                            adoxq %r9, %r15
                            adcxq %r11, %r10

                            mulxq $6, %r8, %r9
                            mulxq $7, %rdi, %r11
                            adoxq %r8, %r10
                            adcxq %r9, %r12
                            adoxq %rdi, %r12
                            adcxq %r11, %r13
                            adoxq $3, %r13

                            movq 16($1), %rdx
                            mulxq 0($2), %r8, %r9
                            mulxq 8($2), %rdi, %r11
                            adcxq %r8, %r15
                            adoxq %r9, %r10
                            adcxq %rdi, %r10
                            adoxq %r11, %r12

                            mulxq 16($2), %r8, %r9
                            mulxq 24($2), %rdi, %r14
                            adcxq %r8, %r12
                            adoxq %r9, %r13
                            adcxq %rdi, %r13
                            adoxq $3, %r14
                            adcxq $3, %r14

                            movq %r15, %rdx
                            mulxq $8, %rdx, %r8
                            mulxq $4, %r8, %r9
                            mulxq $5, %rdi, %r11
                            adoxq %r8, %r15
                            adcxq %r9, %r10
                            adoxq %rdi, %r10
                            adcxq %r11, %r12

                            mulxq $6, %r8, %r9
                            mulxq $7, %rdi, %r11
                            adoxq %r8, %r12
                            adcxq %r9, %r13
                            adoxq %rdi, %r13
                            adcxq %r11, %r14
                            adoxq $3, %r14

                            movq 24($1), %rdx
                            mulxq 0($2), %r8, %r9
                            mulxq 8($2), %rdi, %r11
                            adcxq %r8, %r10
                            adoxq %r9, %r12
                            adcxq %rdi, %r12
                            adoxq %r11, %r13

                            mulxq 16($2), %r8, %r9
                            mulxq 24($2), %rdi, %r15
                            adcxq %r8, %r13
                            adoxq %r9, %r14
                            adcxq %rdi, %r14
                            adoxq $3, %r15
                            adcxq $3, %r15

                            movq %r10, %rdx
                            mulxq $8, %rdx, %r8
                            mulxq $4, %r8, %r9
                            mulxq $5, %rdi, %r11
                            adoxq %r8, %r10
                            adcxq %r9, %r12
                            adoxq %rdi, %r12
                            adcxq %r11, %r13

                            mulxq $6, %r8, %r9
                            mulxq $7, %rdi, %rdx
                            adoxq %r8, %r13
                            adcxq %r9, %r14
                            adoxq %rdi, %r14
                            adcxq %rdx, %r15
                            adoxq $3, %r15

                            movq %r12, 0($0)
                            movq %r13, 8($0)
                            movq %r14, 16($0)
                            movq %r15, 24($0)
                            \"
    ";
