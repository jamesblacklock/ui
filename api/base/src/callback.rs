use std::{
    cell::{RefCell, Cell},
    rc::Rc,
};
use super::{Component, HostAbi};


enum CallbackInner<C: Component> {
	Empty,
	HostAbi(C::Abi),
	Native(Box<dyn Fn(&mut C)>),
}

impl <C: Component> CallbackInner<C> {
	fn call(&self, c: Rc<RefCell<C>>) {
		match self {
			CallbackInner::Empty => {},
			CallbackInner::HostAbi(abi) => abi.call(),
			CallbackInner::Native(f) => f(&mut c.borrow_mut()),
		}
	}
}

impl <C: Component> Default for CallbackInner<C> {
	fn default() -> Self {
		CallbackInner::Empty
	}
}

impl <C: Component> std::fmt::Debug for CallbackInner<C> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			CallbackInner::Empty      => write!(fmt, "Empty"),
			CallbackInner::HostAbi(abi) => write!(fmt, "HostAbi({abi:?})"),
			CallbackInner::Native(_)  => write!(fmt, "Native(Box<dyn Fn()>)"),
		}
	}
}

pub struct Callback<C: Component>(Rc<Cell<CallbackInner<C>>>);

impl <C: Component> Clone for Callback<C> {
	fn clone(&self) -> Self {
		Callback(self.0.clone())
	}
}

impl <C: Component> Default for Callback<C> {
	fn default() -> Self {
		Callback(Rc::new(Cell::new(CallbackInner::Empty)))
	}
}

impl <C: Component, F: 'static + Fn(&mut C)> From<&'static F> for Callback<C> {
	fn from(f: &'static F) -> Self {
		Callback(Rc::new(Cell::new(CallbackInner::Native(Box::new(f)))))
	}
}

impl <C: Component> std::fmt::Debug for Callback<C> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		let item = self.0.take();
		write!(fmt, "Callback({:?})", item)?;
		self.0.set(item);
		Ok(())
	}
}

impl <C: Component> Callback<C> {
	fn call(&self, c: Rc<RefCell<C>>) {
		let f = self.0.take();
		f.call(c);
		self.0.set(f);
	}
    pub fn from_abi(abi: C::Abi) -> Callback<C> {
        Callback(Rc::new(Cell::new(CallbackInner::HostAbi(abi))))
    }
}

impl <C: Component + 'static> Callback<C> {
	pub fn bind(&self, c: &Rc<RefCell<C>>) -> BoundCallback {
		BoundCallback::new(self, c)
	}
}

#[derive(PartialEq)]
enum CallbackId {
    Empty,
    HostAbi(usize),
    Native(usize),
}

trait BoundCallbackTrait: std::fmt::Debug {
	fn call(&self);
	fn clone_box(&self) -> Box<dyn BoundCallbackTrait>;
    fn id(&self) -> CallbackId;
}

#[derive(Debug, Clone)]
struct BoundCallbackImpl<C: Component> {
	callback: Callback<C>,
	component: Rc<RefCell<C>>,
}

impl <C: Component + 'static> BoundCallbackTrait for BoundCallbackImpl<C> {
	fn call(&self) {
		self.callback.call(self.component.clone())
	}
	fn clone_box(&self) -> Box<dyn BoundCallbackTrait> {
		Box::new(BoundCallbackImpl { callback: self.callback.clone(), component: self.component.clone() })
	}
    fn id(&self) -> CallbackId {
        let c = self.callback.0.take();
        let result = match &c {
            CallbackInner::Empty => CallbackId::Empty,
            CallbackInner::HostAbi(f) => CallbackId::HostAbi(f.id()),
            CallbackInner::Native(f) => CallbackId::Native(unsafe { std::mem::transmute(f) }),
        };
        self.callback.0.set(c);
        result
    }
}

#[derive(Debug)]
pub struct BoundCallback(Box<Box<dyn BoundCallbackTrait>>);

impl BoundCallback {
	fn new<C: Component + 'static>(callback: &Callback<C>, component: &Rc<RefCell<C>>) -> BoundCallback {
		BoundCallback(Box::new(Box::new(BoundCallbackImpl {
			callback: callback.clone(),
			component: component.clone(),
		})))
	}

	pub unsafe fn leak(self) -> usize {
		std::mem::transmute(Box::leak(self.0))
	}

	pub unsafe fn restore(ptr: usize) -> Self {
		BoundCallback(Box::from_raw(std::mem::transmute(ptr)))
	}

	pub unsafe fn ptr(self) -> (Self, usize) {
		let ptr = self.leak();
		(Self::restore(ptr), ptr)
	}

	pub fn call(&self) {
		self.0.call();
	}
}

impl Clone for BoundCallback {
	fn clone(&self) -> Self {
		BoundCallback(Box::new(self.0.clone_box()))
	}
}

impl PartialEq for BoundCallback {
    fn eq(&self, other: &BoundCallback) -> bool {
        self.0.id() == other.0.id()
    }
}