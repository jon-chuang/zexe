use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("assembly.rs");

    let macro_string = generate_macro_string(25);

    fs::write(
        &dest_path,
        macro_string
    ).unwrap();

    println!("cargo:rerun-if-changed=src/fields/assembly_string_gen.rs");
    println!("cargo:rerun-if-changed=build.rs");
}

// fn generate_asm_string (limbs: usize) -> &'static str {
//     const wrap = 8 * limbs;
//
//     let asm_string ="";
//     for i in 0..limbs {
//         if i == 0 {
//             asm_string = format!("{}{}", asm_string,
//                 "
//                 movq 0($1), %rdx
//                 xorq $4, $4
//
//                 mulxq 0($2), 8($0), 16($0)
//                 mulxq 8($2), %rax, 24($0)
//                 adcxq %rax, 16($0)
//                 mulxq 16($2), %rax, 0($0)
//                 adcxq %rax, 24($0)
//                 mulxq 24($2), %rax, %rdi
//                 adcxq %rax, 0($0)
//                 adcxq $4, %rdi                 // %rdi is carry1
//                 "
//             );
//         } else {
//             let mut temp = ""
//             for j in 0..limbs-1 {
//                 temp_inner = format!(
//                     "
//                     mulxq 0($2), %rax, %rbx
//                     adcxq %rax, 16($0)
//                     adoxq %rbx, 24($0)
//                     ", , , );
//                 temp = format!("{}{}", temp, temp_inner);
//             }
//             temp = format!("{}{}", temp, "
//                 adoxq %rbx, %rdi
//             ");
//         }
//         let mut temp = format!("
//             movq $5, %rdx
//             mulq {}($0), %rdx               // wrapping_mul
//             ",
//         );
//
//         let mut temp = "
//         mulxq 0($3), %rax, %rbx
//         adcxq 8($0), %rax              // put junk in rax
//         adoxq %rbx, 16($0)
//         ";
//         for j in 1..$limbs {
//             temp_inner = format!("
//                 mulxq {}($3), %rax, %rbx
//                 adcxq %rax, {}($0)
//                 adoxq %rbx, {}($0)
//                 ", (j+i)*8 % wrap, (j+i+1)*8 % wrap, (j+i+2)*8 % wrap
//             );
//             temp = format!("{}{}", temp, temp_inner);
//         }
//         temp = format!("{}{}", temp, "
//             adcxq %rbx, 8($0)
//             adoxq %rdi, 8($0)
//             "
//         );
//         asm_string = format!(asm_string, temp);
//     }
//     asm_string
// }

fn generate_macro_string (max_limbs:usize) -> std::string::String {
    let mut macro_string = String::from(
    "macro_rules! generate_asm_mul {
        ($limbs:expr) => {
            match $limbs {
    ");
    for i in 2..(max_limbs+1) {
        let limb_specialisation = format!(
    "           {} => {{
            }},

    ", i);
        macro_string = format!("{}{}", macro_string, limb_specialisation);
    }
    macro_string = format!("{}{}", macro_string,
            "x => panic!(\"Unexpected invalid number of limbs {:?}\", x)
        }
");
    for i in 2..(max_limbs+1) {
        let function_string = format!(
"
fn mul_asm_{} (a: [u64; $limbs], b: [u64; $limbs]) -> [u64; $limbs] {{
    const ZERO: u64 = 0;
    let mut result = MaybeUninit::<[u64; $limbs]>::uninit();
    unsafe {{
        asm!({}
            :
            // : \"r\"(result.as_mut_ptr()),
            //   \"r\"(&a), \"r\"(&b),
            //   \"m\"(ZERO),
            //   \"m\"(P::MODULUS.0[0]),
            //   \"m\"(P::MODULUS.0[1]),
            //   \"m\"(P::MODULUS.0[2]),
            //   \"m\"(P::MODULUS.0[3]),
            //   \"m\"(P::INV)
            // : \"rdx\", \"rdi\", \"r8\", \"r9\", \"r10\", \"r11\", \"r12\", \"r13\", \"r14\", \"r15\", \"cc\", \"memory\"
            : \"r\"(result.as_mut_ptr()),
              \"r\"(&a), \"r\"(&b), \"r\"(&P::MODULUS.0),
              \"m\"(ZERO),
              \"m\"(P::INV)
            : \"rax\", \"rbx\", \"rdx\", \"rds\", \"rdi\", \"cc\", \"memory\"
        );
    }}
    let r = unsafe {{ result.assume_init() }};
    r
}}
"             , i, ASM_STRING); //generate_asm_string(i));

        macro_string = format!("{}{}", macro_string, function_string);
    }
    macro_string = format!("{}{}", macro_string, "
    }
}");
    macro_string
}


const ASM_STRING :&'static str = "
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
