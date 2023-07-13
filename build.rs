use std::{env, fmt::Write, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tuple.gen.rs");

    fs::write(dest_path, code()).unwrap();

    print!("cargo:rerun-if-changed=build.rs");
}

fn code() -> String {
    let mut code = String::new();
    code.write_str(&tuple_refs_code_gen()).unwrap();

    code
}

fn tuple_refs_code_gen() -> String {
    let mut code = String::new();
    for i in 1..=11 {
        let types_def = make_types_def_list(i);
        let tuple_of_refs_def = make_tuple(i, |t| format!("&'a T{}", t));
        let tuple_of_slices_def = make_tuple(i, |t| format!("&'a [T{}]", t));
        let tuple_of_index_def = make_tuple(i, |t| format!("&self.{}[index]", t));

        let tuple_impl = format!("\
            impl<'a, {types_def}> TupleOfSliceToTupleOfElementRef<{tuple_of_refs_def}> for {tuple_of_slices_def} {{\n\
                #[inline(always)]
                fn as_refs_tuple(self, index: usize) -> {tuple_of_refs_def} {{\n\
                    {tuple_of_index_def}\n\
                }}\n\
            }}\n"
        );

        code.write_str(&tuple_impl).unwrap();
    }

    code
}

fn make_types_def_list(last_type_index: usize) -> String {
    (0..=last_type_index).into_iter()
        .map(|t| format!("T{}", t))
        .collect::<Vec<_>>()
        .join(", ")
}

fn make_tuple(last_type_index: usize, type_index_to_elem_init: fn(usize) -> String) -> String {
    let tuple = (0..=last_type_index).into_iter()
        .map(type_index_to_elem_init)
        .collect::<Vec<_>>()
        .join(", ");
    format!("( {} )", tuple)
}