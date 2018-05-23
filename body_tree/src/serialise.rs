use std::path::PathBuf;
use tree;

pub fn load<P: Into<PathBuf>>(path: P) -> tree::BodyTree {
    unimplemented!()
}

pub fn save<P: Into<PathBuf>>(path: P, tree: tree::BodyTree) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
