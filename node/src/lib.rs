use std::sync::mpsc::{Sender, Receiver, channel};
use authentic_rtree::esmtree::PartionManager;
use structopt::StructOpt;
use types::hash_value::HashValue;
use authentic_rtree::shape::Rect;

pub struct MockChain {
    esmt: PartionManager<f64, 2, 17>,
}

impl MockChain {
    pub fn new() -> Self {
        Self { 
            esmt: PartionManager::new(Rect { _max: [100.0f64, 100.0], _min: [0.0f64, 0.0] }, 1),
        }
    }

    pub fn insert(&mut self, key: String, loc: [f64;2], hash: HashValue) {
        self.esmt.insert(key, loc, hash);
    }

    pub fn delete(&mut self, key: String) {
        self.esmt.delete(&key);
    }

    pub fn update(&mut self, key: String, nloc: [f64;2]) {
        self.esmt.update(&key, nloc);
    }

    pub fn batch_insert(&mut self, data: Vec<(String, [f64;2], HashValue)>) {
        self.esmt.batch_insert(data);
    }

    pub fn hashes(&self) -> Vec<Option<HashValue>>{
        self.esmt.get_hashes()
    }
}

#[derive(Debug, Clone)]
pub enum Request {
    INSERT(String, [f64;2], HashValue),
    DELETE(String),
    UPDATE(String, [f64;2]),
    BATCHINSERT(Vec<(String, [f64;2], HashValue)>),
    QUIT,
}

pub struct Response {
    pub hashes: Vec<Option<HashValue>>,
}

pub struct ClientEnd {
    reqSender: Sender<Request>,
    resReceiver: Receiver<Response>,
}

impl ClientEnd {
    pub fn new(sender: Sender<Request>, recv: Receiver<Response>) -> Self {
        Self {
            reqSender: sender,
            resReceiver: recv,
        }
    }

    pub fn send(&self, req: Request) -> Result<(), std::sync::mpsc::SendError<Request>> {
        self.reqSender.send(req)
    }

    pub fn recv(&self) -> Result<Response, std::sync::mpsc::RecvError> {
        self.resReceiver.recv()
    }
}

pub struct ServerEnd {
    reqReceiver: Receiver<Request>,
    resSender: Sender<Response>,
}

impl ServerEnd {
    pub fn new(sender: Sender<Response>, recv: Receiver<Request>) -> Self {
        Self {
            reqReceiver: recv,
            resSender: sender,
        }
    }

    pub fn send(&self, res: Response) -> Result<(), std::sync::mpsc::SendError<Response>> {
        self.resSender.send(res)
    }

    pub fn recv(&self) -> Result<Request, std::sync::mpsc::RecvError> {
        self.reqReceiver.recv()
    }
}

pub fn generate_channel() -> (ClientEnd, ServerEnd) {
    let (req_sender, req_receiver) = channel();
    let (res_sender, res_receiver) = channel();
    (ClientEnd::new(req_sender, res_receiver), ServerEnd::new(res_sender, req_receiver))
}

#[derive(StructOpt, Debug)]
pub struct NodeArg {
    #[structopt(short = "t", long)]
    pub test: String,
    #[structopt(short = "s", long, default_value = "10000")]
    pub size: usize,
}