(w => {
class _Int {
	static __default() {
		return new _Int(0);
	}
	constructor(arg, value) {
		if(arg != null) {
			value = arg;
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
		} else {
			this.value = value;
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
	constructor(arg, unit, value) {
		if(arg != null) {
			unit = arg?.unit ?? 'px';
			value = arg?.value ?? arg ?? 0;
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
	constructor(arg, brushType, value) {
		if(arg != null) {
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
			brushType = arg.brushType;
			value = arg.value;
		}

		Object.defineProperty(this, 'brushType', { value: brushType, enumerable: true });
		Object.defineProperty(this, 'value', { value: value, enumerable: true });
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

class _Alignment {
	static __default() {
		return new _Alignment(false);
	}
	constructor(arg, value) {
		if(arg == null) {
			this.value = value;
			return;
		}
		switch(String(arg).toLowerCase()) {
			case 'start': this.value = 'start'; break;
			case 'center': this.value = 'center'; break;
			case 'end': this.value = 'end'; break;
			default: this.value = 'stretch'; break;
		}
	}
	css() {
		return this.value;
	}
	jsValue() {
		return this.value;
	}
	flatJsValue() {
		return this.value;
	}
}

class _Float {
	static __default() {
		return new _Float(0);
	}
	constructor(arg, value) {
		if(arg != null) {
			value = arg;
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
		} else {
			this.value = value;
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
	constructor(arg, value) {
		this.value = arg != null ? !!arg : value;
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
	constructor(arg, value) {
		this.value = arg != null ? String(arg) : value;
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
	Object.defineProperty(it, '__dirty', {writable: true, value: false});
	Object.defineProperty(it, '__onCommit', {writable: true, value: onCommit});
	Object.defineProperty(it, '__animationFrame', {writable: true, value: null});
	Object.defineProperty(it, '__isData', {value: true});
	Object.defineProperty(it, 'commit', {value: commit.bind(it)});
	Object.defineProperty(it, 'flatJsValue', {value: flatJsValue.bind(it)});

	function commit() {
		let dirty = applyChanges(this);
		if(dirty && this.__onCommit) {
			this.__onCommit();
		}
		// console.log('dirty:', dirty);
		return dirty;
	}

	function flatJsValue() {
		return this.__collection.map(e => e.flatJsValue());
	}

	function mapValues(method) {
		return function(...args) {
			return method.apply(this.__collection.map(e => e.jsValue()), args);
		};
	}

	function applyChanges(target) {
		let changes = Object.entries(target.__changes);
		let dirty = target.__dirty || changes.length > 0;
		for(let [key, value] of changes) {
			target.__collection[key] = value;
		}
		for(let key in target.__collection) {
			if(target.__changes[key] == null && target.__collection[key].__isData) {
				dirty = target.__collection[key].commit() || dirty;
			}
		}
		target.__changes = {};
		target.__dirty = false;
		// console.log('dirty:', dirty);
		return dirty;
	}

	function setDirty(target) {
		applyChanges(target);
		target.__dirty = true;
	}

	function push(...args) {
		setDirty(this);
		commitNextFrame.call(this);
		return Array.prototype.push.apply(
			this.__collection,
			args.map(e => coerce(e, type.itemType, this.__onCommit))
		);
	}

	function pop() {
		setDirty(this);
		commitNextFrame.call(this);
		return Array.prototype.pop.call(this.__collection)?.flatJsValue();
	}

	function splice(start, deleteCount, ...items) {
		setDirty(this);
		commitNextFrame.call(this);
		return Array.prototype.splice.call(
			this.__collection,
			start,
			deleteCount,
			...items.map(e => coerce(e, type.itemType, this.__onCommit))
		).map(e => e.flatJsValue());
	}

	function shift() {
		setDirty(this);
		commitNextFrame.call(this);
		return Array.prototype.shift.call(this.__collection)?.flatJsValue();
	}

	function unshift(...args) {
		setDirty(this);
		commitNextFrame.call(this);
		return Array.prototype.unshift.apply(
			this.__collection,
			args.map(e => coerce(e, type.itemType, this.__onCommit))
		);
	}

	function reverse(...args) {
		setDirty(this);
		commitNextFrame.call(this);
		Array.prototype.reverse.call(this.__collection);
		return this;
	}

	function sort(compareFn) {
		setDirty(this);
		commitNextFrame.call(this);
		Array.prototype.sort.call(this.__collection, (a,b) => compareFn(a.flatJsValue(),b.flatJsValue()));
		return this;
	}

	function copyWithin(...args) {
		setDirty(this);
		commitNextFrame.call(this);
		Array.prototype.copyWithin.apply(this.__collection, args);
		return this;
	}

	function fill(value, start, end) {
		setDirty(this);
		commitNextFrame.call(this);
		Array.prototype.fill.call(
			this.__collection,
			coerce(value, type.itemType, this.__onCommit),
			start,
			end,
		);
		return this;
	}
	
	function commitNextFrame() {
		w.cancelAnimationFrame(this.__animationFrame);
		this.__animationFrame = w.requestAnimationFrame(() => this.commit()); // IMMEDIATE COMMIT
	}

	let methods = {
		at: mapValues(Array.prototype.at),
		concat: mapValues(Array.prototype.concat),
		entries: mapValues(Array.prototype.entries),
		every: mapValues(Array.prototype.every),
		filter: mapValues(Array.prototype.filter),
		find: mapValues(Array.prototype.find),
		findIndex: mapValues(Array.prototype.findIndex),
		findLast: mapValues(Array.prototype.findLast),
		findLastIndex: mapValues(Array.prototype.findLastIndex),
		flatMap: mapValues(Array.prototype.flat),
		flatMap: mapValues(Array.prototype.flatMap),
		forEach: mapValues(Array.prototype.forEach),
		includes: mapValues(Array.prototype.includes),
		indexOf: mapValues(Array.prototype.indexOf),
		join: mapValues(Array.prototype.join),
		keys: mapValues(Array.prototype.keys),
		lastIndexOf: mapValues(Array.prototype.lastIndexOf),
		map: mapValues(Array.prototype.map),
		reduce: mapValues(Array.prototype.reduce),
		reduceRight: mapValues(Array.prototype.reduceRight),
		slice: mapValues(Array.prototype.slice),
		some: mapValues(Array.prototype.some),
		toLocaleString: mapValues(Array.prototype.toLocaleString),
		toString: mapValues(Array.prototype.toString),
		values: mapValues(Array.prototype.values),

		// mutators
		push,
		pop,
		splice,
		shift,
		unshift,
		reverse,
		sort,
		copyWithin,
		fill,
	};

	return new Proxy(it, {
		get(target, i) {
			if(target[i] !== undefined) {
				return target[i];
			}
			if(i === 'length') {
				return target.__collection.length;
			} else if(methods[i] !== undefined) {
				return methods[i].bind(target);
			}
			i = Number(i);
			if(Number.isNaN(i) || i<0 || i>=target.__collection.length) { return; }
			return (target.__changes[i|0] ?? target.__collection[i|0]).jsValue();
		},
		set(target, i, value) {
			if(i == '__onCommit') {
				target.__onCommit = value;
				return true;
			}

			i = Number(i);
			if(Number.isNaN(i) || i<0 || i>=target.__collection.length) { return; }

			value = coerce(value, target.__type.itemType, onCommit);
			if(equals(value.jsValue(), target.__collection[i|0].jsValue())) {
				delete target.__changes[i|0];
			} else if(target.__dirty) {
				target.__collection[i|0] = value;
			} else {
				target.__changes[i|0] = value;
			}
			commitNextFrame.call(target);
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
					yield [i, j];
				}
			};
		} else if(collection.constructor == Number) {
			this.iter = function*() {
				for(let i = 0; i<collection; i++) {
					yield [i, coerce(i, this.__type.itemType)];
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
		case _Alignment:        return 'Alignment';
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
		if(onCommit == null) {
			throw new Error('should not happen');
		} else if(!onCommit.patched) {
			onCommit = onCommit.bind(null, this);
			onCommit.patched = true;
			// console.log('you should only see this once per component.', type);
		}

		if(type.constructor != _Object) {
			values = type;
			type = deriveType(type);
		}
		if(values == null || typeof values != 'object') {
			values = {};
		}
		
		Object.defineProperty(this, '__type', {value: type});
		Object.defineProperty(this, '__changes', {writable: true, value: {}});
		Object.defineProperty(this, '__props', {writable: true, value: {}});
		Object.defineProperty(this, '__isData', {value: true});
		Object.defineProperty(this, '__neverCommitted', {writable: true, value: true});
		Object.defineProperty(this, '__onCommit', {writable: true, value: onCommit});
		Object.defineProperty(this, '__animationFrame', {writable: true, value: null});
		Object.defineProperty(this, '__blockCommitRecursion', {writable: true, value: 0});

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
					value = coerce(value, this.__type.props[key], this.__onCommit);
					if(equals(value.jsValue(), this.__props[key]?.jsValue())) {
						delete this.__changes[key];
					} else {
						this.__changes[key] = value;
					}
					w.cancelAnimationFrame(this.__animationFrame);
					this.__animationFrame = w.requestAnimationFrame(() => this.commit()); // IMMEDIATE COMMIT
				},
			});

			if(key in values) {
				this.__props[key] = coerce(values[key], this.__type.props[key], this.__onCommit);
			} else {
				this.__props[key] = type.props[key].__default(this.__onCommit);
			}
		}
		Object.seal(this);
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

	commit(alwaysRedraw) {
		// console.log(this.flatJsValue());
		// console.log(this.__changes);
		if(this.__blockCommitRecursion) {
			throw new Error("this shouldn't happen");
			// return true;
		}
		this.__blockCommitRecursion = true;

		let changes = Object.entries(this.__changes);
		let dirty = changes.length > 0 || this.__neverCommitted;
		for(let [key, value] of changes) {
			this.__props[key] = value;
		}
		for(let key in this.__props) {
			if(this.__changes[key] == null && this.__props[key].__isData) {
				dirty = this.__props[key].commit() || dirty;
			}
		}
		this.__changes = {};
		if(alwaysRedraw || dirty && this.__onCommit) {
			this.__onCommit();
		}
		// console.log('dirty:', dirty);
		this.__blockCommitRecursion = false;
		this.__neverCommitted = false;
		return dirty;
	}
}

w.UI = w.UI || {
	__types: {
		_Int,
		_Float,
		_Boolean,
		_String,
		_Length,
		_Brush,
		_Iter,
		_Iterator,
		_Alignment,
		_Array,
		_Object,
		_ObjectInstance,
	},
	__begin(p) {
		p.__l = null;
		p.__e = p.__e || [];
	},
	__ctx(p, e) {
		e.__ctx = {
			parent: p.__ctx,
			width: new w.UI.__types._Length(null, 'px', e.clientWidth),
			height: new w.UI.__types._Length(null, 'px', e.clientHeight),
		};
	},
	__get(p, t, i, c, h) {
		let l = p.__e;
		if(p.__g != null) {
			l = p.__e[p.__g];
			p.__i = i;
		}
		if(!l[i]) {
			if(t == null) {
				l[i] = document.createTextNode("");
			} else if(t.constructor == Function) {
				let d = t(p, null, i, h, false);
				l[i].__d = d;
			} else {
				l[i] = document.createElement(t);
			}
		}
		let e = l[i];
		if(c != null) {
			e.textContent = c;
		}
		return e;
	},
	__in(p, t, i, c, h) {
		let e = w.UI.__get(p, t, i, c, h);
		w.UI.__begin(e);
		w.UI.__ctx(p, e);
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
	__out(p, t, i, c, h) {
		let e = w.UI.__get(p, t, i, c, h);
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
	},
	__fixLayout(e, growLayout) {
		if(e.style.alignSelf == "stretch") {
			if(e.parentElement.style.flexDirection == "row") {
				e.style.height = "";
			} else {
				e.style.width = "";
			}
		}
		if(growLayout) {
			if(e.parentElement.style.flexDirection == "row") {
				e.style.width = "fit-content";
			} else {
				e.style.height = "fit-content";
			}
		}
		e.style.minWidth = e.style.maxWidth = (e.style.width == "0px" ? "" : e.style.width);
		e.style.minHeight = e.style.maxHeight = (e.style.height == "0px" ? "" : e.style.height);
		e.style.flexBasis = "auto";
	}
};
})(window);