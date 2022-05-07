(w => {
w.Thing = w.Thing || {
	__begin(p) {
		p.__l = null;
		p.__e = p.__e || [];
	},
	__get(p, t, i, c) {
		let l = p.__e;
		if(p.__g != null) {
			l = p.__e[p.__g];
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
	__endGroup(p, n) {
		let l = p.__e[p.__g];
		for(let i = n; i<l.length; i++) {
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