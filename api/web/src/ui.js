	(async function(w, d, wasm) {
		let __stringHandler = null;

		const wasm_imports = {
			runtime: {
				__send_string(ptr, len) {
					let str = uiPriv.getStringFromWasm(ptr, len);
					if(__stringHandler == null) {
						return uiPriv.addToHeap(str);
					} else {
						__stringHandler(str);
					}
				},
				__send_bool(value) {
					return uiPriv.addToHeap(!!value);
				},
				__send_f32(value) {
					return uiPriv.addToHeap(value);
				},
				__console_log(ptr, len) {
					const message = uiPriv.getStringFromWasm(ptr, len);
					console.log(message);
				},
				__create_text_node(ptr, len) {
					const content = uiPriv.getStringFromWasm(ptr, len);
					const element = d.createTextNode(content);
					return uiPriv.addToHeap(element);
				},
				__create_element(ptr, len) {
					const tag = uiPriv.getStringFromWasm(ptr, len);
					const element = d.createElement(tag);
					return uiPriv.addToHeap(element);
				},
				__next_sibling(node) {
					return uiPriv.addToHeap(uiPriv.getHeapNode(node).nextSibling);
				},
				__insert_before(node, insert, reference) {
					uiPriv.getHeapNode(node).insertBefore(
						uiPriv.getHeapNode(insert),
						uiPriv.getHeapNode(reference, true));
				},
				__append_child(node, child) {
					uiPriv.getHeapNode(node).appendChild(uiPriv.getHeapNode(child));
				},
				__first_child(node) {
					return uiPriv.addToHeap(uiPriv.getHeapNode(node).firstChild);
				},
				__remove(node) {
					uiPriv.getHeapNode(node).remove();
				},
				__set_text_content(node, ptr, len) {
					uiPriv.getHeapNode(node).textContent = uiPriv.getStringFromWasm(ptr, len);
				},
				__set_style(node, pptr, plen, vptr, vlen) {
					const prop = uiPriv.getStringFromWasm(pptr, plen)
					const value = uiPriv.getStringFromWasm(vptr, vlen)
					uiPriv.getHeapNode(node).style[prop] = value;
				},
				__heap_object_as_bool(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Boolean) {
						return Number(object)|0;
					}
					return -1|0;
				},
				__heap_object_stage_string(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == String) {
						return uiPriv.stageString(object)|0;
					}
					return -1|0;
				},
				__heap_object_load_string(dest) {
					uiPriv.putStagedStringIntoWasm(dest);
				},
				__heap_object_as_f32(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Number) {
						return object;
					}
					return Number.NaN;
				},
				__heap_object_is_function(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					return object?.constructor == Function;
				},
				__heap_object_call_function(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Function) {
						object();
					}
				},
				__heap_object_is_array(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					return object?.constructor == Array;
				},
				__heap_object_get_property(ptr, keyptr, keylen) {
					const object = uiPriv.getHeapObject(ptr);
					if(!(object instanceof Object)) {
						return 0;
					}
					let key = uiPriv.getStringFromWasm(keyptr, keylen);
					if(object[key] == null) {
						return 0;
					}
					return uiPriv.addToHeap(object[key]);
				},
				__heap_object_drop(ptr) {
					uiPriv.dropFromHeap(ptr);
				},
			},
		};

		TYPE_SANITIZERS = {
			Length(value) {
				if(value?.constructor === Number) {
					return `${value}px`;
				} else if(value?.constructor === String) {
					const re = /^(\d*\.\d+|\d+)(px|in|cm|mm)$/;
					if(value.match(re) != null) {
						return value;
					}
				}
				return null;
			},
			Brush(value) {
				throw new Error("unimplemented!");
			},
			Alignment(value) {
				throw new Error("unimplemented!");
			},
			Int(value) {
				if(value?.constructor === Number) {
					return value|0;
				} else if(value?.constructor === String) {
					return parseInt(value)|0;
				}
				return null;
			},
			Float(value) {
				value = parseFloat(value);
				if(value == null || Number.isNaN(value)) {
					return null
				}
				return value;
			},
			String(value) {
				if(value?.constructor === String) {
					return value;
				}
				return null;
			},
			Boolean(value) {
				return !!value;
			},
			Callback(value, ctx) {
				if(value?.constructor === Function) {
					return value.bind(ctx);
				}
				return null;
			},
		}

		class Callback {
			constructor(component, key) {
				this.component = component;
				this.key = key;
			}
		}

		class Iterable {
			constructor(component, key, type) {
				this.component = component;
				this.key = key;
				this.type = type;
			}
		}

		const uiPriv = {
			Callback,
			Iterable,

			components: {},
			heap: [null], // 0 is the null pointer; we don't want anything at index 0
			wasm: WebAssembly.instantiateStreaming(fetch(wasm), wasm_imports),
			decoder: new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }),
			encoder: new TextEncoder('utf-8'),

			getStringFromWasm(ptr, len) {
				if(len == 0) {
					return "";
				}
				const buffer = new Uint8Array(this.memory.buffer).subarray(ptr, ptr + len);
				return uiPriv.decoder.decode(buffer);
			},
			stageString(string) {
				this.stagedString = uiPriv.encoder.encode(string);
				return this.stagedString.length;
			},
			putStagedStringIntoWasm(ptr) {
				const buffer = new Uint8Array(this.memory.buffer).subarray(ptr, ptr + this.stagedString.length);
				buffer.set(this.stagedString);
				delete this.stagedString;
			},
			addToHeap(item) {
				uiPriv.heap.push(item);
				return uiPriv.heap.length - 1;
			},
			dropFromHeap(ptr) {
				let result = uiPriv.heap[ptr];
				delete uiPriv.heap[ptr];
				return result;
			},
			getHeapObject(ptr) {
				return uiPriv.heap[ptr];
			},
			getHeapNode(ptr, optional) {
				let node = uiPriv.heap[ptr];
				if(optional && node == null) {
					return null;
				} else if(!(node instanceof Node)) {
					throw new Error(`"${node}" is not a node`);
				}
				return node;
			},
			sanitizeProps(props, propsDef, ctx) {
				let sanitized = {};
				for(let key in propsDef) {
					let result = null;
					if(propsDef[key] instanceof Array) {
						if(props[key] instanceof Array) {
							result = props[key]
								.map(e => this.sanitize(e, propsDef[key][0], ctx))
								.filter(e => e != null);
						} else {
							result = null;
						}
					} else if(propsDef[key] instanceof Object) {
						throw new Error('unimplemented!');
					} else if(propsDef[key] != null) {
						result = this.sanitize(props[key], propsDef[key], ctx);
					}
					if(result != null) {
						sanitized[key] = result;
					}
				}
				return sanitized;
			},
			sanitize(value, type, ctx) {
				return TYPE_SANITIZERS[type]?.(value, ctx);
			},
			getProperty(component, getter, name, type) {
				if(type instanceof Array) {
					return new Iterable(component, name, type[0]);
				} else if(type == 'Callback') {
					return () => this.dispatch(component, name);
				}
				return uiPriv.dropFromHeap(getter(component.ptr));
			},
			setProperty(component, setter, value, type) {
				if(type == 'Callback') {
					value = value.bind(component);
				}
				setter(component.ptr, this.addToHeap(value));
				cancelAnimationFrame(component.animationFrame);
				component.animationFrame = requestAnimationFrame(() => component.render());
			},
			dispatch(component, name) {
				console.log('dispatch:', name);
			}
		}

		const UI = {
			async loadComponent(name) {
				if(uiPriv.components[name] != null) {
					return uiPriv.components[name];
				}

				const wasm = await uiPriv.wasm;

				function getExport(exportName) {
					if(!(wasm.instance.exports[exportName] instanceof Function)) {
						throw new Error(`"${name}" is not a valid component`);
					}
					return wasm.instance.exports[exportName]
				}

				const componentPriv = {
					__attach_to_element: getExport(`${name}__attach_to_element`),
					__render_component:  getExport(`${name}__render_component`),
					__update_component:  getExport(`${name}__update_component`),
					__new_component:     getExport(`${name}__new_component`),
					__get_props_json:    getExport(`${name}__get_props_json`),
				};
				
				let propsDef;
				{
					let json;
					__stringHandler = str => { json = str };
					componentPriv.__get_props_json();
					__stringHandler = null;
					propsDef = JSON.parse(json);
				}

				

				Class = eval(`(class ${name} { constructor(props) { this.__constructor(props) } })`);
				Class.prototype.__constructor = function(props) {
					props = uiPriv.sanitizeProps(props, propsDef, this);
					const ptr = componentPriv.__new_component(uiPriv.addToHeap(props));
					Object.defineProperty(this, 'ptr', { value: ptr });
				}
				Class.prototype.propsDef = propsDef;
				Class.prototype.attachToElement = function(element) {
					if(typeof element == 'string') {
						element = d.getElementById(element);
					}
					if(!(element instanceof HTMLElement)) {
						console.error('failed to attach component to DOM: invalid element received');
						return;
					}
					let eptr = uiPriv.addToHeap(element);
					componentPriv.__attach_to_element(this.ptr, eptr);
					this.render();
				}
				Class.prototype.render = function() {
					componentPriv.__update_component(this.ptr);
					componentPriv.__render_component(this.ptr);
				}
				
				for(let key in propsDef) {
					let get;
					if(propsDef[key] == 'Callback') {
						let getter = getExport(`${name}__call_${key}`);
						get = function() {
							return () => getter(this.ptr);
						}
					} else {
						let getter = getExport(`${name}__get_${key}`);
						get = function() {
							return uiPriv.getProperty(this, getter, key, type);
						}
					}
					let setter = getExport(`${name}__set_${key}`);
					let type = propsDef[key];
					Object.defineProperty(Class.prototype, key, {
						enumerable: true,
						get,
						set(value) {
							uiPriv.setProperty(this, setter, value, propsDef[key]);
						},
					});
				}

				uiPriv.components[name] = Class;
				return Class;
			}
		};
		w.UI = UI;
		w.__uiPriv = uiPriv;
		uiPriv.wasm.then(wasm => { uiPriv.memory = wasm.instance.exports.memory });
	})