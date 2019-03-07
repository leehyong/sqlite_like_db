
mod test{
    use std::process::{Command, Stdio};
    use std::io::Write;
    #[test]
    fn t1(){
        let mut child = Command::new("./target/debug/lhyDB")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        {
            let mut stdin = child.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all("insert 1 user1 person1@example.com\n".as_bytes()).expect("Failed to write to stdin");
            stdin.write_all("select\n".as_bytes()).expect("Failed to write to stdin");
            stdin.write_all(".exit\n".as_bytes()).expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");
        assert_eq!(String::from_utf8_lossy(&output.stdout),
                   "db > Executed.\ndb > (1, user1, person1@example.com)\nExecuted.\ndb > "
        );

    }

    #[test]
    fn t2(){
        let mut child = Command::new("./target/debug/lhyDB")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        {
            let mut stdin = child.stdin.as_mut().expect("Failed to open stdin");
            for i in (1..=91){
                let mut str1 = format!("insert {} user{} person{}@example.com\n", i, i, i);
                stdin.write_all(str1.as_bytes()).expect("Failed to write to stdin");
            }
            stdin.write_all(".exit\n".as_bytes()).expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");
        let mut outstr = &mut output.stdout.rsplit(|b|*b==b'\n').into_iter().take(2).skip(1);
        match outstr.next() {
            Some(s) =>{
                assert_eq!(String::from_utf8_lossy(s),
                           "db > Error: Table full."
                );
            }
            _ =>{
                unreachable!()
            }
        }



    }
}