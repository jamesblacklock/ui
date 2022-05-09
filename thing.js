(w => {
w.Thing = w.Thing || {
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