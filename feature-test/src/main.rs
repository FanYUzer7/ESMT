use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::time::{Instant};
use feature_test::xy2d;
use rand::{Rng, thread_rng};
use types::hash_value::{HashValue};
use authentic_rtree::node::{HilbertSorter};
use authentic_rtree::shape::Rect;
use types::test_utils::{calc_hash, num_hash};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct FeatureArgs {
    #[structopt(short = "h", long)]
    pub history: Option<String>,
}

struct Cmd {
    func: String,
    params: Vec<String>,
}

impl Cmd {
    pub fn from_cmdline(cmd: String) -> Self {
        let iter = cmd.split_ascii_whitespace().collect::<Vec<_>>();
        if iter.len() == 0 {
            return Self::empty();
        }
        let mut iter = iter.into_iter().map(|s| s.to_string());
        let func = iter.next().unwrap();
        let params = iter.collect();
        Self {
            func,
            params,
        }
    }

    fn empty() -> Self {
        Self {
            func: "empty".to_string(),
            params: vec![]
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

#[derive(Serialize, Deserialize)]
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

    pub fn from_args(args: FeatureArgs) -> Self {
        let service = if let Some(path) = args.history {
            let mut f = File::open(path).unwrap();
            let mut s = "".to_string();
            let _ = f.read_to_string(&mut s);
            serde_json::from_str(&s).unwrap()
        } else {
            Self::default()
        };
        for (i,p) in service.points.iter().enumerate() {
            println!("point[{}] = {:?}",i, *p);
        }
        service
    }

    pub fn save(&self) {
        let mut f = File::create("./service.his").unwrap();
        let his = serde_json::to_string(self).unwrap();
        let _ = f.write_all(his.as_bytes());
    }
}

struct RemoteProcedure {
    m: HashMap<String, Box<dyn Fn(Cmd, &mut CliService)>>,
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
        // let func_hanle = FuncHandler::new(handle);
        // self.m.insert(method, Box::new(func_hanle));
        self.m.insert(method, Box::new(handle));
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
        methods.insert("test".to_string(), |cmd, service| {
            test_efficient(cmd, service);
        });
        methods.insert("bit".to_string(), |cmd, service| {
            num_bits(cmd, service);
        });
        methods.insert("hilbert".to_string(), |cmd, service| {
            hilbert_sort(cmd, service);
        });
        methods
    }

    pub fn call(&self, cmd: Cmd, service: &mut CliService) {
        let func = self.m.get(cmd.pattern()).unwrap();
        // (*func).call(cmd, service);
        func(cmd, service)
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let args: FeatureArgs = FeatureArgs::from_args();
    let mut rl = Editor::<()>::new()?;
    let _ = rl.load_history("history.txt");
    let mut service = CliService::from_args(args);
    let method = RemoteProcedure::default();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let cmd = Cmd::from_cmdline(line);
                if cmd.pattern() == "quit" {
                    break;
                } else if cmd.pattern() == "empty" {
                    continue;
                } else {
                    method.call(cmd, &mut service);
                }
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
    service.save();
    rl.save_history("history.txt")
}

fn help(_cmd: Cmd, _service: &mut CliService) {
    println!("help");
    println!("sort [params]            hilbert sort idx");
    println!("hash [params]            calculate the root hash");
    println!("pack [tot] [cap]         pack node");
    println!("area [point...]          calculate area");
    println!("update [point] [x] [y]   update point location");
    println!("bit [num]                update point location");
    println!("hilbert [order]          generate hilbert matrix");
    println!("quit                     quit the program");
}

fn pack(cmd: Cmd, _service: &mut CliService) {
    assert_eq!(cmd.argc(), 2, "please input correct params");
    let args = cmd.params();

    let total = usize::from_str(&args[0]).unwrap();
    let cap = usize::from_str(&args[1]).unwrap();
    // assert!(total <= cap *cap, "total nums must no greater than cap2");
    let down = (cap + 1) / 2;
    let full_pack_size = cap;
    let full_pack_remain = total % cap;
    let full_pack_cnt = total / cap;
    let res = {
        if full_pack_remain == 0 {
            vec![full_pack_size; full_pack_cnt as usize]
        } else {
            if full_pack_cnt >= 1 {
                let mut res = vec![full_pack_size; (full_pack_cnt - 1) as usize];
                if full_pack_remain < down {
                    res.push(full_pack_size + full_pack_remain - down);
                    res.push(down);
                } else {
                    res.push(full_pack_size);
                    res.push(full_pack_remain);
                }
                res
            } else {
                let res = vec![full_pack_remain];
                res
            }
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
                            let _ = brack_cnt;
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
    let parse_str = args[0].clone();
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

    let old = service.points[p].clone();
    service.points[p] = [x,y];
    println!("point {}: {:?} ---> {:?}",p, old,[x,y]);
}

fn test_efficient(cmd: Cmd, _service: &mut CliService) {
    let args = cmd.params();
    let num = i32::from_str(&args[0]).unwrap();

    let mut rng = thread_rng();
    let mut dur1 = 0_u128;
    let mut dur2 = 0_u128;
    for _ in 0..1000 {
        let mut vec1 = {
            let mut v = vec![];
            for _ in 0..num {
                v.push(rng.gen_range(0..10000));
            }
            v
        };
        let mut vec2 = VecDeque::new();
        vec2.extend(vec1.clone());
        let (n1, n2) = (99999,99998);
        let ins1 = Instant::now();
        vec1.insert(0, n1);
        vec1.insert(0, n2);
        dur1 += ins1.elapsed().as_nanos();
        println!("{:?}", vec1);

        let ins2 = Instant::now();
        vec2.push_front(n1);
        vec2.push_front(n2);
        dur2 += ins2.elapsed().as_nanos();
        println!("{:?}", vec2);
    }
    println!("insert: {}ns, extend: {}ns", dur1 / 1000, dur2 / 1000);
}

fn num_bits(cmd: Cmd, _service: &mut CliService) {
    let args = cmd.params();
    let num = usize::from_str(&args[0]).unwrap();
    let bits = {
        let mut v = vec![];
        let mut n = num;
        for _ in 0..3 {
            v.push(n % 2);
            n >>= 1;
        }
        v.reverse();
        v
    };
    println!("{:?}",bits);
}

fn hilbert_sort(cmd: Cmd, _service: &mut CliService) {
    let args = cmd.params();
    let order = u32::from_str(&args[0]).unwrap();
    let n = 2i32.pow(order);
    for x in 1..=n {
        for y in (1..=n).rev() {
            print!("{}, ", xy2d(n, x, y));
        }
        println!()
    }
}