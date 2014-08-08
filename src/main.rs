#![feature(macro_rules)]
use std::comm; 
macro_rules! with(
    ($($element:ident),*) => (
        vec![$($element.subscribe(),)*]
        )
    )
struct LastMessage<T> {
    last_value: T,
    recveiver : comm::Receiver<T>
}
impl<T: Send + Clone> LastMessage<T>{
    fn new(r: comm::Receiver<T>, default: T) -> LastMessage<T>{
        LastMessage{ last_value: default, recveiver: r}
    }
    fn get(&mut self) -> T {
        let result = self.recveiver.try_recv();
        match result {
            Ok(value) =>{
                self.last_value = value;
            }
            _         => ()
        }
        self.last_value.clone()
    }
}
fn message<T: Send + Clone>(default: T) -> (comm::Sender<T>,
                                            LastMessage<T>){
    let (s,r) = comm::channel();
    let r = LastMessage::new(r, default);
    (s,r)
}


trait ReactiveContainer<T> {
    fn set_value(&mut self,value : T);
    fn get_value(&self) -> T;
    fn get_sender(&self) -> Vec<comm::Sender<T>>;
    fn notify(&self);
    fn value(&mut self,value : T){
        self.set_value(value);
        self.notify();
    }
    fn subscribe(&mut self)-> LastMessage<T>;

}
struct VarSignal<T>{
    value : T,
    sender:Vec<comm::Sender<T>>
}
impl<T:Send+Clone> VarSignal<T> {
    fn new(v: T) -> VarSignal<T>{
        VarSignal { value :v ,sender: Vec::new()}
    }
}
impl <T :Send + Clone> ReactiveContainer<T> for VarSignal<T>{
    fn set_value(&mut self, value :T){
        self.value = value;
        self.notify();
    }
    fn get_value(&self) -> T{
        self.value.clone()
    }
    fn get_sender(&self) -> Vec<comm::Sender<T>>{
        self.sender.clone()
    }
    fn notify(&self){
        for sender in self.get_sender().iter(){
            sender.send(self.get_value());
        }
    }
    fn subscribe(&mut self)-> LastMessage<T>{
        let (s,r): (Sender<T>,LastMessage<T>) = message(self.get_value());
        self.sender.push(s);
        r
    }
}
struct Signal<'r,T,R>{
    a : LastMessage<T>,
    b : LastMessage<T>,
    f : |T,T|:'r -> R
}
impl<'r,T: Send + Clone,R> Signal<'r,T,R> {
    fn new(a: LastMessage<T>, b: LastMessage<T>, f: |T,T|->R) -> Signal<T,R> {
        let signal = Signal{a:a,b:b,f:f};
        signal
    }
    fn get(&mut self) -> R {
        let v1 =self.a.get();
        let v2 =self.b.get();
        (self.f)(v1,v2)
    }
}

struct Foo<'a,A,B>{
    my_function: |A|:'a -> B
}

impl<'a,A,B> Foo<'a,A,B>{
    fn new(f: |A| -> B) -> Foo<A,B>{
        Foo {my_function:f}
    }
}
fn main(){
    let add = |a: i32, b| { a + b };
    let mut v1 = VarSignal::new(10i32);
    let mut v2 = VarSignal::new(20i32);
    let mut s = Signal::new(v1.subscribe(),v2.subscribe(),add);
    println!("{}",s.get());
    v1.set_value(100);
    println!("{}",s.get());
    v2.set_value(200);
    println!("{}",s.get());

}
