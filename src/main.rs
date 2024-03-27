use std::rc::Rc;

mod tokenizer;

// enum Error {
//     TokenizerError(usize)
// }

use crate::tokenizer::tokenize;

fn main() {
    let s = r#"'aaa' "bbb" `ccc`"#.to_string();
    // let s = r#"124 0x2fFF 'aha' "hehe""#.to_string();

    println!("str: {:#?}", s);

    let x = tokenize(Rc::new(s));

    println!("got: {:#?}", x);
}

