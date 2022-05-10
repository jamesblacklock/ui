(w => {
class _Int {
	static __default() {
		return new _Int(0);
	}
	constructor(value) {
		value = value ?? 0;
		if(value.constructor == String) {
			this.value = parseInt(value);
		} else if(value.constructor == Number) {
			this.value = value|0;
		} else {
			this.value = 0;
		}
		if(Number.isNaN(this.value)) {
			this.value = 0;
		}
	}
	jsValue() {
		return this.value;
	}
	flatJsValue() {
		return this.jsValue();
	}
}

class _Length {
	static __default() {
		return new _Length(0);
	}
	constructor(arg) {
		let unit = arg?.unit ?? 'px';
		let value = arg?.value ?? arg ?? 0;
		
		if(value.value != null) {
			value = value;
		}
		if(value.constructor == String) {
			if(value.match(/^(\d*\.\d+|\d+)in$/)) {
				unit = 'in';
			} else if(value.match(/^(\d*\.\d+|\d+)cm$/)) {
				unit = 'cm';
			}
			value = parseFloat(value);
		} else if(value.constructor == Number) {
			value = value;
		} else {
			value = 0;
		}
		if(Number.isNaN(this.value)) {
			value = 0;
		}
		Object.defineProperty(this, 'unit', { value: unit, enumerable: true });
		Object.defineProperty(this, 'value', { value: value, enumerable: true });
	}
	jsValue() {
		return `${this.value}${this.unit}`;
	}
	flatJsValue() {
		return `${this.value}${this.unit}`;
	}
}

class _Float {
	static __default() {
		return new _Float(0);
	}
	constructor(value) {
		value = value ?? 0;
		if(value.constructor == String) {
			this.value = parseFloat(value);
		} else if(value.constructor == Number) {
			this.value = value;
		} else {
			this.value = 0;
		}
		if(Number.isNaN(this.value)) {
			this.value = 0;
		}
	}
	jsValue() {
		return this.value;
	}
	flatJsValue() {
		return this.jsValue();
	}
}

class _Boolean {
	static __default() {
		return new _Boolean(false);
	}
	constructor(value) {
		this.value = !!value;
	}
	jsValue() {
		return this.value;
	}
	flatJsValue() {
		return this.jsValue();
	}
}

class _String {
	static __default() {
		return new _String("");
	}
	constructor(value) {
		this.value = String(value);
	}
	jsValue() {
		return this.value;
	}
	flatJsValue() {
		return this.jsValue();
	}
}

class _Object {
	constructor(props) {
		this.props = props;
	}
	__default(onCommit) {
		return new _ObjectInstance(this, null, onCommit);
	}
}

function deriveType(t) {
	if(t == null) {
		return _Int;
	} else if(t == String || t.constructor == String) {
		return _String;
	} else if(t == Boolean || t.constructor == Boolean) {
		return _Boolean;
	} else if(t == Number || t.constructor == Number) {
		return _Float;
	} else {
		let types = {};
		for(key in t) {
			types[key] = deriveType(t[key]);
		}
		return new _Object(types);
	}
}

function coerce(v, t, onCommit) {
	if(v == t) {
		return t.__default();
	} else if(t.constructor == _Object) {
		return new _ObjectInstance(t, v, onCommit);
	} else {
		return new t(v);
	}
}

function equals(l, r) {
	if(l == null || r == null) {
		return l == r;
	}
	if(isPrimitive(l) || isPrimitive(r)) {
		return l === r;
	}
	if(l.__isData) {
		l = l.flatJsValue();
	}
	if(r.__isData) {
		r = r.flatJsValue();
	}
	// if(l.constructor == Array || r.constructor == Array) {
	// 	if(l.constructor != r.constructor || l.length != r.length) {
	// 		return false;
	// 	}
	// 	for(let i=0; i<l.length; i++) {
	// 		if(!equals(l[i], r[i])) {
	// 			return false;
	// 		}
	// 	}
	// } else {
		let keys = new Set([...Object.keys(l), ...Object.keys(r)]);
		for(let key of keys) {
			if(!equals(l[key], r[key])) {
				return false;
			}
		}
	// }
	return true;
}

function isPrimitive(object) {
	return object.constructor == Number || object.constructor == String || object.constructor == Boolean;
}

class _ObjectInstance {
	constructor(type, values, onCommit) {
		if(type.constructor != _Object) {
			values = type;
			type = deriveType(type);
		}
		Object.defineProperty(this, '__type', {enumerable: false, writable: true, value: type});
		Object.defineProperty(this, '__changes', {enumerable: false, writable: true, value: {}});
		Object.defineProperty(this, '__props', {enumerable: false, writable: true, value: {}});
		Object.defineProperty(this, '__isData', {enumerable: false, writable: true, value: true});
		Object.defineProperty(this, '__onCommit', {enumerable: false, writable: true, value: onCommit.bind(null, this)});
		Object.defineProperty(this, '__ready', {enumerable: false, writable: true, value: false});
		for(let key in type.props) {
			Object.defineProperty(this, key, {
				enumerable: true,
				get() {
					let result;
					if(this.__changes[key] !== undefined) {
						result = this.__changes[key].jsValue();
					} else {
						result = this.__props[key].jsValue();
					}
					return result;
				},
				set(value) {
					value = coerce(value, this.__type.props[key], () => this.commit());
					if(equals(value.jsValue(), this.__props[key].jsValue())) {
						delete this.__changes[key];
					} else {
						this.__changes[key] = value;
					}
					// this.commit();
				},
			});
			this.__props[key] = type.props[key].__default(() => this.commit());
		}
		Object.seal(this);
		if(values != null) {
			for(let key in values) {
				this[key] = values[key];
			}
		}
		this.__ready = true;
		this.commit();
	}

	jsValue() {
		return this;
	}

	flatJsValue() {
		let object = {};
		for(let key in this.__props) {
			object[key] = (this.__changes[key] ?? this.__props[key]).flatJsValue();
		}
		return object;
	}

	commit() {
		if(!this.__ready) {
			return false;
		}
		let changes = Object.entries(this.__changes);
		let dirty = changes.length > 0;
		this.__ready = false;
		for(let [key, value] of changes) {
			this.__props[key] = value;
		}
		for(let key in this.__props) {
			if(this.__changes[key] == null && this.__props[key].__isData) {
				dirty = this.__props[key].commit() || dirty;
			}
		}
		this.__changes = {};
		if(dirty && this.__onCommit) {
			this.__onCommit();
		}
		this.__ready = true;
		console.log('dirty:', dirty);
		return dirty;
	}
}

w.Thing = w.Thing || {
	__types: {
		_Int,
		_Float,
		_Boolean,
		_String,
		_Length,
		_Object,
		_ObjectInstance,
	},
	__iter(n) {
		if(n.constructor == Array) {
			return n.entries();
		} else if(n.constructor == Number) {
			let i = 0;
			return {
				[Symbol.iterator]() {
					return {
						next() {
							if(i < n) {
								let res = { done: false, value: [i, i] };
								i++;
								return res;
							} else {
								return { done: true, value: undefined };
							}
						}
					};
				}
			};
		} else {
			return { next: () => ({ done: true, value: undefined }) }
		}
	},
	__begin(p) {
		p.__l = null;
		p.__e = p.__e || [];
		p.__ctx = p.__ctx || {};
	},
	__ctx(p, e) {
		e.__ctx = {
			parent: p.__ctx,
		};
	},
	__get(p, t, i, c) {
		let l = p.__e;
		if(p.__g != null) {
			l = p.__e[p.__g];
			p.__i = i;
		}
		if(!l[i]) {
			l[i] = t ? document.createElement(t) : document.createTextNode("");
		}
		let e = l[i];
		if(c != null) {
			e.textContent = c;
		}
		return e;
	},
	__in(p, t, i, c) {
		let e = w.Thing.__get(p, t, i, c);
		w.Thing.__begin(e);
		w.Thing.__ctx(p, e);
		if(e.__in) {
			p.__l = e;
			return e;
		}
		if(p.__l) {
			if(p.__l.nextSibling) {
				p.insertBefore(e, p.__l.nextSibling);
			} else {
				p.appendChild(e);
			}
		} else if(p.children.length > 0) {
			p.insertBefore(e, p.children[0]);
		} else {
			p.appendChild(e);
		}
		p.__l = e;
		e.__in = true;
		return e;
	},
	__out(p, t, i, c) {
		let e = w.Thing.__get(p, t, i, c);
		if(e.__in) {
			e.remove();
			e.__in = false;
		}
	},
	__beginGroup(p, i) {
		p.__g = i;
		if(!p.__e[i]) {
			p.__e[i] = [];
		}
	},
	__endGroup(p) {
		let l = p.__e[p.__g];
		for(let i = p.__i; i<l.length; i++) {
			l[i].remove();
			l[i].__in = false;
		}
		p.__g = null;
	},
	__event(e, n, d, c) {
		e.__events = e.__events || {};
		if(e.__events[n]?.unbound == c) {
			return;
		}
		if(e.__events[n]) {
			e.removeEventListener("click", e.__events[n].bound);
		}
		if(c) {
			e.__events[n] = { unbound: c, bound: c.bind(d) };
			e.addEventListener("click", e.__events[n].bound);
		}
	}
};
})(window);