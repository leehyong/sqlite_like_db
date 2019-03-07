#![allow(unused)]
use std::mem::size_of;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{self, stdin, Read, BufRead, Result as IoResult, Error, ErrorKind, stdout, Write};
//use bytes::{Buf, BufMut, Bytes, BytesMut};

struct InputBuffer{
//    buffer:BytesMut,
    buffer:String,
}

const EXIT_CODE:i8 = -1;
const META_CODE:i8 = 0;

const PAGE_SIZE:usize = 1024;
const ROW_SIZE:usize = size_of::<Row>();
const TABLE_MAX_PAGES:usize = 5;
const ROWS_PER_PAGE:usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS:usize = TABLE_MAX_PAGES * ROWS_PER_PAGE;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
enum ExecuteResult{
    EXECUTE_TABLE_FULL,
    EXECUTE_SUCCESS
}
impl Default for ExecuteResult{
    fn default() ->Self{
        ExecuteResult::EXECUTE_SUCCESS
    }
}
struct Page{
 rows:Vec<Row>
}
impl Page{
    fn new() ->Self{
        Self{rows:Vec::with_capacity(TABLE_MAX_ROWS)}
    }
}


struct Table{
    pages:Vec<Page>,
    num_rows:usize
}

impl Table{
    fn new() ->Self{
        Self{pages:Vec::with_capacity(TABLE_MAX_PAGES), num_rows:0}
    }

    fn is_full(&self) -> bool{
        self.num_rows == TABLE_MAX_ROWS
    }
    fn row_slot(&self, row_num:usize) -> usize{
        row_num / ROWS_PER_PAGE
    }

    fn serialize_row(&mut self, row:&Row) ->ExecuteResult{
        if !self.is_full(){
            let page = self.row_slot(self.num_rows);
            if self.pages.len() != page + 1{
                self.pages.push(Page::new());
            }
            self.pages.get_mut(page).unwrap().rows.push(row.clone());
            self.num_rows += 1;
            return ExecuteResult::EXECUTE_SUCCESS
        }
        ExecuteResult::EXECUTE_TABLE_FULL
    }
    
//    fn deserialize_row(&self, row_num:usize) -> &Row{
//        let row_seq = row_num % ROWS_PER_PAGE;
//        self.pages.get(self.row_slot(row_num)).unwrap().rows.get(row_seq).unwrap()
//    }
}

impl InputBuffer{
    fn new()->Self{
        Self{buffer:String::with_capacity(64)}
    }

    fn read_input(&mut self){
        let mut s = String::new();
        match stdin().read_line(&mut s){
            Ok(n) if n > 0 =>{
                self.buffer = s.trim().to_lowercase();
            }
            Ok(_) => {
                unreachable!()
            }
            Err(e) =>{
                eprintln!("{}", e);
            }
        }
    }
    fn parse(&self) -> IoResult<i8>{
        if self.buffer.len() <= 0{
            return Err(Error::new(ErrorKind::Other, "Error reading input"));
        }
        use MetaCommandResult::*;
        if self.buffer.as_bytes()[0] == b'.'{
            match self.do_meta_command() {
                META_COMMAND_SUCCESS =>{
                    if self.buffer.eq_ignore_ascii_case(".exit")
                        ||self.buffer.eq_ignore_ascii_case(".e")
                        ||self.buffer.eq_ignore_ascii_case(".quit")
                        ||self.buffer.eq_ignore_ascii_case(".q"){
                        return Ok(EXIT_CODE)
                    }
                    return Ok(META_CODE)
                }
                META_COMMAND_UNRECOGNIZED_COMMAND =>{
                    return Err(Error::new(ErrorKind::Other, format!("Unrecognized command '{}'", self.buffer)));
                }
            }
        }
        return Ok(1)
    }

    fn do_meta_command(&self) -> MetaCommandResult{
        let ss = self.buffer.split_whitespace().into_iter().take(1).next().unwrap();
        if ss == ".tables"
            || ss == ".xxxxx"
            || ss == ".exit"
            || ss == ".q"
            || ss == ".e"
            || ss == ".quit"
        {
            return MetaCommandResult::META_COMMAND_SUCCESS
        }
        MetaCommandResult::META_COMMAND_UNRECOGNIZED_COMMAND
    }
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
enum StatementType{
    STATEMENT_NONE,
    STATEMENT_INSERT,
    STATEMENT_SELECT
}

impl Default for StatementType{
    fn default() -> Self{
        StatementType::STATEMENT_NONE
    }
}

#[derive(Debug, Default, Clone)]
struct Row{
    id :u32,
    name :String,
    email :String
}

#[derive(Clone, Debug, Default)]
struct Statement{
 typ:StatementType,
 row:Row
}

impl Statement{
    fn new() -> Self{
        Statement::default()
    }

    fn prepare_statement(&mut self, buf:&InputBuffer)->PrepareResult{
        use PrepareResult::*;
        if buf.buffer.starts_with("insert"){
            self.typ = StatementType::STATEMENT_INSERT;
            let sss = buf.buffer.split_whitespace().into_iter().skip(1).collect::<Vec<&str>>();
            if sss.len() < 3{
                return PREPARE_SYNTAX_ERROR
            }
            match sss[0].parse::<u32>(){
                Ok(id) => {
                    self.row.id = id;
                },
                Err(e) => {
                    println!("{}({})", sss[0], e);
                    return PREPARE_SYNTAX_ERROR;
                }
            };
            self.row.name = sss[1].to_string();
            self.row.email = sss[2].to_string();
            return PREPARE_SUCCESS;
        }
        if buf.buffer.starts_with("select"){
            self.typ = StatementType::STATEMENT_SELECT;
            return PREPARE_SUCCESS;
        }
        PREPARE_UNRECOGNIZED_STATEMENT

    }

    fn handle_statement(&mut self, buf :&InputBuffer, table:&mut Table){
        use PrepareResult::*;
        match self.prepare_statement(buf) {
            PREPARE_SUCCESS => {
                match self.execute_statement(table) {
                    ExecuteResult::EXECUTE_SUCCESS =>{
                        println!("Executed.");
                    }
                    ExecuteResult::EXECUTE_TABLE_FULL =>{
                        println!("Error: Table full.");
                    }
                }
            },
            PREPARE_UNRECOGNIZED_STATEMENT =>{
                println!("Unrecognized keyword at start of '{}'", buf.buffer)
            }
            _ =>{

            }
        }
    }
    fn execute_statement(&mut self, table:&mut Table) -> ExecuteResult{
        use StatementType::*;
        match self.typ {
            STATEMENT_INSERT =>{
                return self.execute_insert_statement(table);
            },
            STATEMENT_SELECT =>{
                return self.execute_select_statement(table);
            }
            _ =>{
                return ExecuteResult::EXECUTE_SUCCESS;
            }
        }
    }
    fn execute_insert_statement(&mut self, table:&mut Table) -> ExecuteResult{
        table.serialize_row(&self.row)
    }
    fn execute_select_statement(&self, table:&Table)-> ExecuteResult{
        for page in &table.pages{
            for row in &page.rows{
                print!("{}", &row);
            }
        }
        ExecuteResult::EXECUTE_SUCCESS
    }
}
impl Display for Row{
    fn fmt(&self, f: &mut Formatter) -> FmtResult{
        write!(f, "({}, {}, {})\n", self.id, self.name, self.email)
    }
}

#[allow(non_camel_case_types)]
enum MetaCommandResult{
    META_COMMAND_SUCCESS,
    META_COMMAND_UNRECOGNIZED_COMMAND,
}
#[allow(non_camel_case_types)]
enum PrepareResult{
    PREPARE_SUCCESS,
    PREPARE_SYNTAX_ERROR,
    PREPARE_UNRECOGNIZED_STATEMENT,
}

fn print_prompt(){
    stdout().write("db > ".as_bytes()).unwrap();
    stdout().flush().unwrap();
}

fn main() {
    let mut buf = InputBuffer::new();
    use PrepareResult::*;
    let mut table = Table::new();
//    println!("{}, {}, {}", TABLE_MAX_ROWS, ROWS_PER_PAGE, ROW_SIZE);
    loop{
        print_prompt();
        buf.read_input();
        match buf.parse(){
            Ok(EXIT_CODE) =>{
                break
            }
            Ok(n) =>{
                if n == META_CODE{

                }else{
                    let mut statement = Statement::new();
                    statement.handle_statement(&buf, &mut table);
                }
            }
            Err(e) =>{
                println!("{}", e);
            }
        }
    }
}

