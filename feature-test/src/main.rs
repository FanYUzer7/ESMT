use std::collections::{BTreeSet, HashMap};
use std::str::FromStr;
use rand::{Rng, thread_rng};
use types::hash_value::{ESMTHasher, HashValue};
use MerkleRTree::node::{HilbertSorter};
use MerkleRTree::shape::Rect;
use MerkleRTree::mrtree::MerkleRTree as Tree;
use types::test_utils::{calc_hash, num_hash};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

struct Cmd {
    func: String,
    params: Vec<String>,
}

impl Cmd {
    pub fn from_cmdline(cmd: String) -> Self {
        let mut iter = cmd.split_ascii_whitespace().map(|s| s.to_string());
        let func = iter.next().unwrap();
        let params = iter.collect();
        Self {
            func,
            params,
        }
    }

    #[inline]
    pub fn pattern(&self) -> &String {
        &self.func
    }

    #[inline]
    pub fn params(self) -> Vec<String> {
        self.params
    }

    #[inline]
    pub fn argc(&self) -> usize {
        self.params.len()
    }
}

struct CliService {
    pub points: Vec<[usize; 2]>,
    pub hashes: Vec<HashValue>,
}

impl CliService {
    pub fn default() -> Self {
        let points = vec![
            [1usize, 6],
            [0, 5],
            [3, 2],
            [4, 5],
            [8, 5],
            [2, 8],
            [2, 3],
            [6, 7],
            [8, 0],
            [1, 1]
        ];
        let hashes = (0..10).map(|i| num_hash(i)).collect();
        Self {
            points,
            hashes,
        }
    }
}

struct RemoteProcedure {
    m: HashMap<String, Box<dyn Method>>,
}

trait Method {
    fn call(&self, cmd: Cmd, service: &mut CliService);
}

struct FuncHandler<T>
    where
        T: Fn(Cmd, &mut CliService),
{
    handle: T,
}

impl<F: Fn(Cmd, &mut CliService)> FuncHandler<F> {
    pub fn new(handle: F) -> Self {
        Self {
            handle
        }
    }
}

impl<F: Fn(Cmd, &mut CliService)> Method for FuncHandler<F> {
    fn call(&self, cmd: Cmd, service: &mut CliService) {
        (self.handle)(cmd, service);
    }
}

impl RemoteProcedure {
    pub fn new() -> Self {
        Self {
            m: HashMap::new(),
        }
    }

    pub fn insert<F: Fn(Cmd, &mut CliService) + 'static>(&mut self, method: String, handle: F) {
        let func_hanle = FuncHandler::new(handle);
        self.m.insert(method, Box::new(func_hanle));
    }

    pub fn default() -> Self {
        let mut methods = RemoteProcedure::new();
        methods.insert("help".to_string(), |cmd, service| {
            help(cmd, service);
        });
        methods.insert("pack".to_string(), |cmd, service| {
            pack(cmd, service);
        });
        methods.insert("sort".to_string(), |cmd, service| {
            sort(cmd, service);
        });
        methods.insert("hash".to_string(), |cmd, service| {
            hash(cmd, service);
        });
        methods.insert("area".to_string(), |cmd, service| {
            area(cmd, service);
        });
        methods.insert("update".to_string(), |cmd, service| {
            update(cmd, service);
        });
        methods
    }

    pub fn call(&self, cmd: Cmd, service: &mut CliService) {
        let func = self.m.get(cmd.pattern()).unwrap();
        (*func).call(cmd, service);
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new()?;
    let _ = rl.load_history("history.txt");
    let mut service = CliService::default();
    let method = RemoteProcedure::default();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let cmd = Cmd::from_cmdline(line);
                method.call(cmd, &mut service);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")
}

fn help(_cmd: Cmd, _service: &mut CliService) {
    println!("help");
    println!("sort [params]            hilbert sort idx");
    println!("hash [params]            calculate the root hash");
    println!("pack [tot] [cap]         pack node");
    println!("area [point...]          calculate area");
    println!("update [point] [x] [y]   update point location");
    println!("quit                     quit the program");
}

fn pack(cmd: Cmd, _service: &mut CliService) {
    assert_eq!(cmd.argc(), 2, "please input correct params");
    let args = cmd.params();

    let total = usize::from_str(&args[0]).unwrap();
    let cap = usize::from_str(&args[1]).unwrap();
    assert!(total <= cap *cap, "total nums must no greater than cap2");
    let down = (cap + 1) / 2;
    let mut full_pack_size = cap;
    let full_pack_remain = total % cap;
    let full_pack_cnt = total / cap;
    let res = {
        if full_pack_remain == 0 {
            vec![full_pack_size; full_pack_cnt as usize]
        } else {
            let mut res = vec![full_pack_size; (full_pack_cnt - 1) as usize];
            if full_pack_remain < down {
                res.push(full_pack_size + full_pack_remain - down);
                res.push(down);
            } else {
                res.push(full_pack_size);
                res.push(full_pack_remain);
            }
            res
        }
    };
    println!("packed node {:?}", res);
}

fn sort(cmd: Cmd, service: &mut CliService) {
    assert_eq!(cmd.argc(), 1, "at least one point");
    let mut range = None;
    let mut range_set = vec![];
    let mut stack: Vec<Rect<usize, 2>> = vec![];
    let mut parse_stack = vec![];

    let args = cmd.params();
    let parse_str = args[0].clone();
    let mut brack_cnt = 0;
    for &byte in parse_str.as_bytes() {
        match byte {
            91u8 => { // '['
                parse_stack.push(byte);
                brack_cnt += 1;
            }
            93u8 => { // ']'
                if brack_cnt == 1 {
                    while let Some(ch) = parse_stack.pop() {
                        if ch != 91 {
                            range_set.push(stack.pop().unwrap());
                        } else {
                            brack_cnt -= 1;
                            break;
                        }
                    }
                    break;
                }
                let mut temp_range = None;
                while let Some(ch) = parse_stack.pop() {
                    if ch != 91 {
                        if temp_range.is_some() {
                            let r: &mut Rect<usize, 2> = temp_range.as_mut().unwrap();
                            r.expand(&(stack.pop().unwrap()));
                        } else {
                            temp_range = Some(stack.pop().unwrap());
                        }
                    } else {
                        stack.push(temp_range.unwrap());
                        parse_stack.push(92u8);
                        brack_cnt -= 1;
                        break;
                    }
                }
            }
            48..=57u8 => {
                let idx = (byte - 48) as usize;
                parse_stack.push(byte);
                let rect = Rect::new_point(service.points[idx].clone());
                stack.push(rect.clone());
                if range.is_none() {
                    range = Some(rect);
                } else {
                    let r: &mut Rect<usize, 2> = range.as_mut().unwrap();
                    r.expand(&rect);
                }
            }
            _ => { continue; }
        }
    }
    let sorter = HilbertSorter::<usize, 2, 3>::new(range.as_ref().unwrap());
    for i in range_set.into_iter().rev() {
        let hilbert_idx = sorter.hilbert_idx(&i);
        println!("point [{}] hilbert idx = {}", i, hilbert_idx);
    }
}

fn hash(cmd: Cmd, service: &mut CliService) {
    assert_eq!(cmd.argc(), 1, "please input correct hash set");
    let mut stack = vec![];
    let mut parse_stack = vec![];

    let args = cmd.params();
    let parse_str = args[1].clone();
    for &byte in parse_str.as_bytes() {
        match byte {
            91u8 => { // '['
                parse_stack.push(byte);
            }
            93u8 => { // ']'
                let mut temp_set = BTreeSet::new();
                while let Some(ch) = parse_stack.pop() {
                    if ch != 91 {
                        temp_set.insert(stack.pop().unwrap());
                    } else {
                        stack.push(calc_hash(&temp_set));
                        parse_stack.push(92u8);
                        break;
                    }
                }
            }
            48..=57u8 => {
                let idx = (byte - 48) as usize;
                parse_stack.push(byte);
                stack.push(service.hashes[idx]);
            }
            _ => { continue; }
        }
    }
    assert_eq!(parse_stack.len(), 1, "parse error");
    println!("{:?}", stack.pop().unwrap());
}

fn area(cmd: Cmd, service: &mut CliService) {
    assert!(cmd.argc() >= 1, "at least one point");
    let args = cmd.params();
    let range_set = args
        .iter()
        .map(|s| usize::from_str(s).unwrap())
        .collect::<Vec<_>>();
    let mut range = Rect::<usize, 2>::new_point(service.points[range_set[0]]);
    for i in range_set.iter() {
        range.expand(&Rect::<usize,2>::new_point(service.points[*i]));
    }
    println!("{:?} area = {}",range, range.area());
}

fn update(cmd: Cmd, service: &mut CliService) {
    assert_eq!(cmd.argc(), 3, "please input correct params");
    let args = cmd.params();

    let p = usize::from_str(&args[0]).unwrap();
    let x = usize::from_str(&args[1]).unwrap();
    let y = usize::from_str(&args[2]).unwrap();

    service.points[p] = [x,y];
    println!("point {} ---> {:?}",p, [x,y]);
}