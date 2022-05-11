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
		if(Number.isNaN(value)) {
			value = 0;
		}
		Object.defineProperty(this, 'unit', { value: unit, enumerable: true });
		Object.defineProperty(this, 'value', { value: value, enumerable: true });
	}
	css() {
		return this.jsValue();
	}
	jsValue() {
		return `${this.value}${this.unit}`;
	}
	flatJsValue() {
		return `${this.value}${this.unit}`;
	}
}

const clamp = (num, min, max) => Math.min(Math.max(num ?? 0, min), max);
const hexDoubleDigit = (d) => { let n = parseInt(d, 16); return n << 4 | n }

class _Brush {
	static __default() {
		return new _Brush();
	}
	constructor(arg) {
		let transparent = { brushType: 'color', value: { r: 0, g: 0, b: 0, a: 0 } };

		arg = arg ?? transparent;
		if(arg.constructor == String) {
			if(arg.match(/^#[\da-fA-F]{3}$/)) {
				let r = hexDoubleDigit(arg[1]);
				let g = hexDoubleDigit(arg[2]);
				let b = hexDoubleDigit(arg[3]);
				arg = { brushType: 'color', value: { r, g, b, a: 1 } };
			} else if(arg.match(/^#[\da-fA-F]{4}$/)) {
				let r = hexDoubleDigit(arg[1]);
				let g = hexDoubleDigit(arg[2]);
				let b = hexDoubleDigit(arg[3]);
				let a = hexDoubleDigit(arg[4]) / 255;
				arg = { brushType: 'color', value: { r, g, b, a } };
			} else if(arg.match(/^#[\da-fA-F]{6}$/)) {
				let r = parseInt(arg.slice(1,3), 16);
				let g = parseInt(arg.slice(3,5), 16);
				let b = parseInt(arg.slice(5,7), 16);
				arg = { brushType: 'color', value: { r, g, b, a: 1 } };
			} else if(arg.match(/^#[\da-fA-F]{8}$/)) {
				let r = parseInt(arg.slice(1,3), 16);
				let g = parseInt(arg.slice(3,5), 16);
				let b = parseInt(arg.slice(5,7), 16);
				let a = parseInt(arg.slice(7,9), 16) / 255;
				arg = { brushType: 'color', value: { r, g, b, a } };
			} else {//if(arg.match(/^rgb\(\d\)$/)) {
				arg = transparent;
			}
		} else if(arg.brushType == null) {
			arg = { brushType: 'color', value: arg };
		}

		if(arg.brushType == 'color') {
			arg.value = {
				r: clamp(arg.value?.r, 0, 255)|0,
				g: clamp(arg.value?.g, 0, 255)|0,
				b: clamp(arg.value?.b, 0, 255)|0,
				a: clamp(arg.value?.a, 0, 1),
			};
		} else {
			arg = transparent;
		}

		Object.defineProperty(this, 'brushType', { value: arg.brushType, enumerable: true });
		Object.defineProperty(this, 'value', { value: arg.value, enumerable: true });
	}
	css() {
		return this.jsValue();
	}
	jsValue() {
		return `rgba(${this.value.r},${this.value.g},${this.value.b},${this.value.a})`;
	}
	flatJsValue() {
		return this.jsValue();
	}
}

class _Direction {
	static __default() {
		return new _Direction(false);
	}
	constructor(arg) {
		this.vertical = arg == 'vertical';
	}
	css() {
		return this.vertical ? 'column' : 'row';
	}
	jsValue() {
		return this.vertical ? 'vertical' : 'horizontal';
	}
	flatJsValue() {
		return this.jsValue();
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

class _Iter {
	constructor(itemType) {
		this.itemType = itemType;
	}
	__default() {
		return new _Iterator(this, 0);
	}
}

function _Array(type, arr, onCommit) {
	let it = new _Iterator(type, arr.map(e => coerce(e, type.itemType, onCommit)));
	Object.defineProperty(it, '__changes', {writable: true, value: {}});
	Object.defineProperty(it, '__onCommit', {value: onCommit});
	Object.defineProperty(it, '__isData', {value: true});
	Object.defineProperty(it, 'commit', {value: commit.bind(it)});

	function commit() {
		let changes = Object.entries(this.__changes);
		let dirty = changes.length > 0;
		for(let [key, value] of changes) {
			this.__collection[key] = value;
		}
		for(let key in this.__collection) {
			if(this.__changes[key] == null && this.__collection[key].__isData) {
				dirty = this.__collection[key].commit() || dirty;
			}
		}
		this.__changes = {};
		if(dirty && this.__onCommit) {
			this.__onCommit();
		}
		// console.log('dirty:', dirty);
		return dirty;
	}

	return new Proxy(it, {
		get(target, i) {
			if(target[i] !== undefined) {
				return target[i];
			}
			i = Number(i);
			if(Number.isNaN(i) || i<0 || i>=target.__collection.length) { return; }
			return coerce(target.__collection[i|0], target.__type.itemType).jsValue();
		},
		set(target, i, value) {
			i = Number(i);
			if(Number.isNaN(i) || i<0 || i>=target.__collection.length) { return; }

			value = coerce(value, target.__type.itemType, target.__onCommit);
			if(equals(value.jsValue(), target.__collection[i|0].jsValue())) {
				delete target.__changes[i|0];
			} else {
				target.__changes[i|0] = value;
			}
			target.commit();
		}
	});
}

class _Iterator {
	__isIter = true;
	constructor(type, collection) {
		collection = collection ?? [];
		
		Object.defineProperty(this, '__collection', { enumerable: true, value: collection });
		Object.defineProperty(this, '__type', { enumerable: true, value: type });

		if(collection.constructor == Array) {
			this.iter = function*() {
				for(let [i, j] of collection.entries()) {
					yield [i, j/*coerce(j, this.__type.itemType)*/.jsValue()];
				}
			};
		} else if(collection.constructor == Number) {
			this.iter = function*() {
				for(let i = 0; i<collection; i++) {
					yield [i, coerce(i, this.__type.itemType, this.__onCommit).jsValue()];
				}
			};
		} else {
			this.iter = { next: () => ({ done: true, value: undefined }) }
		}
	}
	jsValue() {
		return this;
	}
	flatJsValue() {
		return this.__collection;
	}
}
class _Object {
	constructor(props) {
		this.props = props;
	}
	__default(onCommit) {
		return new _ObjectInstance(this, null, onCommit);
	}
	flatJsValue() {
		let types = {};
		for(let key in this.props) {
			types[key] = typeToFlatJsValue(this.props[key]);
		}
		return types;
	}
}

function typeToFlatJsValue(t) {
	if(t.constructor == _Iter) {
		return [typeToFlatJsValue(t.itemType)];
	} else if(t.constructor == _Object) {
		return t.flatJsValue();
	}
	switch(t) {
		case _Int:              return 'Int';
		case _Float:            return 'Float';
		case _Boolean:          return 'Boolean';
		case _String:           return 'String';
		case _Length:           return 'Length';
		case _Brush:            return 'Brush';
		case _Direction:        return 'Direction';
		// case _Array:            types[key] = 'e'; break;
		default:                throw new Error('unimplemented');
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
	} else if(t.constructor == _Iter) {
		if(v.constructor == Array) {
			return _Array(t, v, onCommit);
		} else {
			return new _Iterator(t, v);
		}
	} else {
		return new t(v);
	}
}

function equals(l, r) {
	if(l == null || r == null) {
		return l == r;
	}
	if(l.__isIter) {
		l = l.__collection;
	}
	if(r.__isIter) {
		r = r.__collection;
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
		if(onCommit) {
			onCommit = onCommit.bind(null, this);
		}
		Object.defineProperty(this, '__type', {value: type});
		Object.defineProperty(this, '__changes', {writable: true, value: {}});
		Object.defineProperty(this, '__props', {writable: true, value: {}});
		Object.defineProperty(this, '__isData', {value: true});
		Object.defineProperty(this, '__onCommit', {value: onCommit});
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
					value = coerce(value, this.__type.props[key], onCommit);
					if(equals(value.jsValue(), this.__props[key].jsValue())) {
						delete this.__changes[key];
					} else {
						this.__changes[key] = value;
					}
					this.commit();
				},
			});
			this.__props[key] = type.props[key].__default(onCommit);
		}
		Object.seal(this);
		if(values != null) {
			for(let key in values) {
				if(key in this.__props) {
					this[key] = values[key];
				} else {
					console.error('tried to set invalid property:', key);
				}
			}
		}
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
		let changes = Object.entries(this.__changes);
		let dirty = changes.length > 0;
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
		// console.log('dirty:', dirty);
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
		_Brush,
		_Iter,
		_Iterator,
		_Direction,
		_Array,
		_Object,
		_ObjectInstance,
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
		p.__i = -1;
		if(!p.__e[i]) {
			p.__e[i] = [];
		}
	},
	__endGroup(p) {
		let l = p.__e[p.__g];
		for(let i = p.__i+1; i<l.length; i++) {
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