use std::io::Write;
use tinify::tinify;

fn main() {
    match tinify("./testdata/peppers.jpg", 90) {
        Ok(buf) => {
            let mut file = std::fs::File::create("./res.jpg").unwrap();
            file.write_all(&buf).unwrap();
        }
        Err(e) => panic!("{:?}", e),
    }
}
