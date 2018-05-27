use super::Population;
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn load<P: Into<PathBuf>>(path: P) -> Population {
    let path = path.into();
    let f = File::open(&path).expect(&format!("Failed to read file {:?}", path));
    deserialise(f)
}

pub fn save<P: Into<PathBuf>>(path: P, pop: &Population) {
    let path = path.into();
    let f = File::create(&path).expect(&format!("Failed to create file {:?}", path));
    serialise(f, pop)
}

fn deserialise<R: Read>(reader: R) -> Population {
    serde_json::from_reader(reader).expect("Failed to deserialise")
}

fn serialise<W: Write>(writer: W, pop: &Population) {
    serde_json::to_writer(writer, &pop).expect("Failed to serialise");
}

#[cfg(test)]
mod tests {
    use super::{deserialise, serialise};
    use body::def::*;
    use std::cell::RefCell;
    use std::io::Cursor;
    use std::rc::Rc;
    use tree::*;

    #[test]
    fn save_and_load() {
        let tree = {
            let root_shape = new_cuboid((0.5, 2.0, 1.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0));
            let mut t = BodyTree::with_root(Rc::new(RefCell::new(root_shape)));
            let root = t.root();
            t.add_child(
                root,
                Rc::new(RefCell::new(new_cuboid(
                    (1.0, 3.0, 0.1),
                    (0.0, 2.0, 0.0),
                    (1.2, 2.0, 1.0),
                ))),
                Joint::Fixed,
            );
            t
        };

        let vec = Vec::new();
        let mut cursor = Cursor::new(vec);
        let pop = vec![tree];

        let _serialised = serialise(&mut cursor, &pop);
        cursor.set_position(0);
        println!("{}", String::from_utf8(cursor.get_ref().to_vec()).unwrap());
        let deserialised = deserialise(&mut cursor);

        assert_eq!(pop.len(), deserialised.len());
    }
}
