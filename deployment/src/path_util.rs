use std::path::Path;

pub trait CutBase{
    fn cut_base(&self, starting_segment: &str) -> String;
}
impl CutBase for Path{
    fn cut_base(&self, starting_segment: &str) -> String {
        self
            .iter()
            .skip_while(|x|{
                *x != starting_segment
            })
            .map(|s| s.to_str().unwrap())
            .collect::<Vec<_>>()
            .join(&std::path::MAIN_SEPARATOR.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_base() {
        let path = "/home/owo/foo/bar";
        let p: &Path = path.as_ref();
        let relative = p.cut_base("owo");
        assert_eq!(relative, "owo/foo/bar");
    }
}

